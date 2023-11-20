use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use task_options::TaskOptions;
use task_result::TaskResult;
use utils::{value_from_file, value_to_file};

pub mod branch;
pub mod ordered_queued_task;
pub mod queued_task;
pub mod task_options;
pub mod task_ref_inner;
pub mod task_result;
pub mod task_status;

pub const DAGS_DIR: &str = "./bin";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: usize,
    pub function_name: String,
    pub template_args: Value,
    pub options: TaskOptions,
    pub lazy_expand: bool,
    pub is_dynamic: bool,
    pub is_branch: bool,
}

impl Task {
    pub fn execute(
        &self,
        resolved_args: &Value,
        attempt: usize,
        handle_stdout: Box<dyn Fn(String) + Send>,
        handle_stderr: Box<dyn Fn(String) + Send>,
        executable_path: &str,
    ) -> TaskResult {
        if attempt > 1 {
            thread::sleep(self.options.retry_delay);
        }

        let task_id: usize = self.id;
        let function_name = &self.function_name;
        let resolved_args_str = serde_json::to_string(resolved_args).unwrap();
        let in_path = format!("./in_{function_name}_{task_id}.json");
        let out_path = format!("./{function_name}_{task_id}.json");
        let use_timeout = self.options.timeout.is_some();
        let timeout_as_secs = self
            .options
            .timeout
            .unwrap_or(Duration::ZERO)
            .as_secs()
            .to_string();

        value_to_file(resolved_args, &in_path);

        let start = Utc::now();
        let mut child = Command::new(if use_timeout {
            "timeout"
        } else {
            executable_path
        })
        .args(if use_timeout {
            vec![
                "-k",
                &timeout_as_secs,
                &timeout_as_secs,
                executable_path,
                "run",
                "function",
                &function_name,
                &out_path,
                &in_path,
            ]
        } else {
            vec![
                "run",
                "function",
                function_name.as_str(),
                &out_path,
                &in_path,
            ]
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start command");

        let stdout = child.stdout.take().expect("failed to take stdout");
        let stderr = child.stderr.take().expect("failed to take stderr");

        // Spawn a thread to handle stdout
        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = format!("{}\n", line.expect("failed to read line from stdout"));
                handle_stdout(line);
            }
        });

        // Spawn a thread to handle stderr
        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = format!("{}\n", line.expect("failed to read line from stdout"));
                handle_stderr(line);
            }
        });

        let status = child.wait().expect("failed to wait on child");
        let end = Utc::now();

        let timed_out = matches!(status.code(), Some(124));

        // Join the stdout and stderr threads
        stdout_handle.join().expect("stdout thread panicked");
        stderr_handle.join().expect("stderr thread panicked");

        TaskResult {
            task_id,
            result: if status.success() {
                value_from_file(&out_path)
            } else {
                Value::Null
            },
            attempt,
            max_attempts: self.options.max_attempts,
            function_name: function_name.to_string(),
            resolved_args_str,
            success: status.success(),
            started: start.to_rfc3339(),
            ended: end.to_rfc3339(),
            elapsed: end.timestamp() - start.timestamp(),
            premature_failure: false,
            premature_failure_error_str: if timed_out {
                "timed out".into()
            } else {
                "".into()
            },
            is_branch: self.is_branch,
        }
    }
}
