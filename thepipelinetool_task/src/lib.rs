use std::{env, ffi::OsStr, fs, path::PathBuf, process::Command, thread};

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use task_options::TaskOptions;
use task_result::TaskResult;
use thepipelinetool_utils::{spawn, value_from_file, value_to_file};

pub mod branch;
pub mod ordered_queued_task;
pub mod queued_task;
pub mod task_options;
pub mod task_ref_inner;
pub mod task_result;
pub mod task_status;
pub mod temp_queued_task;
pub mod trigger_rule;

fn get_json_dir() -> String {
    env::var("JSON_DIR")
        .unwrap_or("./json/".to_string())
        .to_string()
}

fn get_save_to_file() -> bool {
    env::var("SAVE_TO_FILE")
        .unwrap_or("false".to_string())
        .to_string()
        .parse::<bool>()
        .unwrap()
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub function: String,
    pub template_args: Value,
    pub options: TaskOptions,
    pub lazy_expand: bool,
    pub is_dynamic: bool,
    pub is_branch: bool,
    pub use_trigger_params: bool,
}

impl Task {
    pub fn execute<P, D>(
        &self,
        resolved_args: &Value,
        attempt: usize,
        handle_stdout_log: Box<dyn Fn(String) -> Result<()> + Send>,
        handle_stderr_log: Box<dyn Fn(String) -> Result<()> + Send>,
        take_last_stdout_line: Box<dyn Fn() -> Result<String> + Send>,
        pipeline_path: P,
        tpt_path: D,
        run_id: usize,
    ) -> Result<TaskResult>
    where
        P: AsRef<OsStr>,
        D: AsRef<OsStr>,
    {
        let task_id: usize = self.id;
        let function_name = &self.function;
        let resolved_args_str = serde_json::to_string(resolved_args).unwrap();
        let mut cmd = Command::new(tpt_path);
        cmd.arg(pipeline_path);
        cmd.args(["run", "function", &self.function]);
        cmd.env("run_id", run_id.to_string());

        let out_path: Option<PathBuf> = if get_save_to_file() {
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
            Some(out_path)
        } else {
            cmd.arg(&resolved_args_str);
            None
        };

        if attempt > 1 {
            thread::sleep(self.options.retry_delay);
        }
        let start = Utc::now();

        // TODO store exit code? (coudl allow for 'skipped' status)
        let exit_status = spawn(
            cmd,
            self.options.timeout,
            handle_stdout_log,
            handle_stderr_log,
        );
        let (success, code) = match exit_status {
            Ok(exit_status) => (exit_status.success(), exit_status.code()),
            Err(_) => (false, Some(124)),
        };
        let timed_out = code == Some(124);
        let end = Utc::now();

        let result = match (success, get_save_to_file()) {
            (true, true) => value_from_file(&out_path.unwrap()).unwrap(),
            (true, false) => serde_json::from_str(&take_last_stdout_line().unwrap()).unwrap(),
            (false, _) => Value::Null,
        };

        Ok(TaskResult {
            task_id,
            result,
            attempt,
            max_attempts: self.options.max_attempts,
            name: self.name.clone(),
            function: function_name.clone(),
            resolved_args_str,
            success,
            started: Some(start),
            ended: Some(end),
            elapsed: end.timestamp() - start.timestamp(),
            premature_failure: false,
            premature_failure_error_str: if timed_out { "timed out" } else { "" }.into(),
            is_branch: self.is_branch,
            is_sensor: self.options.is_sensor,
            exit_code: code,
        })
    }
}
