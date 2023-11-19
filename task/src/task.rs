use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utils::{value_from_file, value_to_file};

use crate::{task_options::TaskOptions, task_result::TaskResult};

pub const DAGS_DIR: &str = "./bin";


#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: usize,
    pub function_name: String,
    pub template_args: Value,
    pub options: TaskOptions,
    pub lazy_expand: bool,
    pub is_dynamic: bool,
    pub is_branch: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrderedQueuedTask {
    pub score: usize,
    pub queued_task: QueuedTask,
}

impl Ord for OrderedQueuedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .score
            .cmp(&self.score)
            .then_with(|| other.queued_task.task_id.cmp(&self.queued_task.task_id))
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::max_by(self, other, Ord::cmp)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::min_by(self, other, Ord::cmp)
    }
}

impl PartialOrd for OrderedQueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct QueuedTask {
    pub task_id: usize,
    pub run_id: usize,
    pub dag_name: String,
}


impl Task {
    pub fn execute(
        &self,
        dag_name: String,
        resolved_args: Value,
        attempt: usize,
        handle_stdout: Box<dyn Fn(String) + Send>,
        handle_stderr: Box<dyn Fn(String) + Send>,
    ) -> TaskResult {
        let task_id: usize = self.id;
        let function_name = self.function_name.clone();
        let resolved_args_str = serde_json::to_string_pretty(&resolved_args).unwrap();
        let retry_delay = self.options.retry_delay;
        let max_attempts = self.options.max_attempts;
        let timeout = self.options.timeout;
        let timeout_as_secs = timeout.unwrap_or(Duration::ZERO).as_secs().to_string();
        let is_branch = self.is_branch;

        let in_path = format!("./in_{function_name}_{task_id}.json");
        value_to_file(&resolved_args, &in_path);

        if attempt > 1 {
            thread::sleep(retry_delay);
        }

        let start = Utc::now();
        let out_path = format!("./{function_name}_{task_id}.json");

        let binding = env::current_exe().unwrap().display().to_string();
        let mut ex = binding.as_str();

        let t: String = format!("{DAGS_DIR}/{dag_name}");
        if ex.ends_with("server") || ex.ends_with("worker") {
            ex = &t;
        }

        let use_timeout = timeout.is_some();
        let f = if use_timeout { "timeout" } else { ex };
        let a = if use_timeout {
            vec![
                "-k",
                &timeout_as_secs,
                &timeout_as_secs,
                ex,
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
        };
        let mut command = Command::new(f);
        command
            .args(a)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().expect("failed to start command");

        let stdout = child.stdout.take().expect("failed to take stdout");
        let stderr = child.stderr.take().expect("failed to take stderr");

        let end = Utc::now();

        // Spawn a thread to handle stdout
        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = format!("{}\n", line.expect("failed to read line from stdout"));
                // println!("stdout: {}", &line);
                // let mut accum = stdout_accum_clone.lock().unwrap();
                // accum.push_str(&line);
                // accum.push('\n');

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
            max_attempts,
            function_name,
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
            is_branch,
        }
    }
}
