use std::{
    env,
    ffi::OsStr,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    thread,
    time::Duration,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use task_options::TaskOptions;
use task_result::TaskResult;
use thepipelinetool_utils::{value_from_file, value_to_file};

pub mod branch;
pub mod ordered_queued_task;
pub mod queued_task;
pub mod task_options;
pub mod task_ref_inner;
pub mod task_result;
pub mod task_status;
pub mod trigger_rules;

fn get_json_dir() -> String {
    env::var("JSON_DIR")
        .unwrap_or("./json/".to_string())
        .to_string()
}

fn get_save_to_file() -> bool {
    env::var("SAVE_TO_FILE")
        .unwrap_or("false".to_string())
        .to_string().parse::<bool>().unwrap()
}

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
    pub fn execute<P>(
        &self,
        resolved_args: &Value,
        attempt: usize,
        handle_stdout_log: Box<dyn Fn(String) + Send>,
        handle_stderr_log: Box<dyn Fn(String) + Send>,
        take_last_stdout_line: Box<dyn Fn() -> String + Send>,
        executable_path: P,
    ) -> TaskResult
    where
        P: AsRef<OsStr>,
    {
        let save_to_file = get_save_to_file();

        if attempt > 1 {
            thread::sleep(self.options.retry_delay);
        }
        let task_id: usize = self.id;
        let function_name = &self.function_name;
        let resolved_args_str = serde_json::to_string(resolved_args).unwrap();
        let use_timeout = self.options.timeout.is_some();
        let timeout_as_secs = self
            .options
            .timeout
            .unwrap_or(Duration::ZERO)
            .as_secs()
            .to_string();

        let start = Utc::now();
        let mut cmd = self.create_command(&executable_path, use_timeout);

        self.command_timeout(&mut cmd, &executable_path, use_timeout, &timeout_as_secs);

        let (status, timed_out, result) = if save_to_file {
            let json_dir = get_json_dir();
            let out_path: PathBuf = [&json_dir, &format!("{function_name}_{task_id}_out.json")]
                .iter()
                .collect();
            let in_path: PathBuf = [&json_dir, &format!("{function_name}_{task_id}_in.json")]
                .iter()
                .collect();
            fs::create_dir_all(&json_dir).unwrap();
            value_to_file(resolved_args, &in_path);
            cmd.args([&in_path, &out_path]);

            let (status, timed_out) = self.spawn(cmd, handle_stdout_log, handle_stderr_log);
            if status.success() {
                (status, timed_out, value_from_file(&out_path).unwrap())
            } else {
                (status, timed_out, Value::Null)
            }
        } else {
            cmd.arg(&resolved_args_str);
            let (status, timed_out) = self.spawn(cmd, handle_stdout_log, handle_stderr_log);
            if status.success() {
                let last_line = take_last_stdout_line();
                (status, timed_out, serde_json::from_str(&last_line).unwrap())
            } else {
                (status, timed_out, Value::Null)
            }
        };
        let end = Utc::now();
        let premature_failure_error_str = if timed_out { "timed out" } else { "" }.into();

        TaskResult {
            task_id,
            result,
            attempt,
            max_attempts: self.options.max_attempts,
            function_name: function_name.to_string(),
            resolved_args_str,
            success: status.success(),
            started: start.to_rfc3339(),
            ended: end.to_rfc3339(),
            elapsed: end.timestamp() - start.timestamp(),
            premature_failure: false,
            premature_failure_error_str,
            is_branch: self.is_branch,
        }
    }

    fn create_command<P>(&self, executable_path: &P, use_timeout: bool) -> Command
    where
        P: AsRef<OsStr>,
    {
        if use_timeout {
            Command::new("timeout")
        } else {
            Command::new(executable_path)
        }
    }

    fn command_timeout<P>(
        &self,
        command: &mut Command,
        executable_path: &P,
        use_timeout: bool,
        timeout_as_secs: &str,
    ) where
        P: AsRef<OsStr>,
    {
        if use_timeout {
            command.args(["-k", timeout_as_secs, timeout_as_secs]);
            command.arg(executable_path);
        }

        command.args(["run", "function", &self.function_name]);
    }

    fn spawn(
        &self,
        mut cmd: Command,
        handle_stdout_log: Box<dyn Fn(String) + Send>,
        handle_stderr_log: Box<dyn Fn(String) + Send>,
    ) -> (ExitStatus, bool) {
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to start command");

        let stdout = child.stdout.take().expect("failed to take stdout");
        let stderr = child.stderr.take().expect("failed to take stderr");

        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = format!("{}\n", line.expect("failed to read line from stdout"));
                handle_stdout_log(line);
            }
        });

        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = format!("{}\n", line.expect("failed to read line from stdout"));
                handle_stderr_log(line);
            }
        });

        let status = child.wait().expect("failed to wait on child");
        let timed_out = matches!(status.code(), Some(124));
        stdout_handle.join().expect("stdout thread panicked");
        stderr_handle.join().expect("stderr thread panicked");

        (status, timed_out)
    }
}
