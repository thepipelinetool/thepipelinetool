use std::{env, ffi::OsStr, fs, path::PathBuf, thread, time::Duration};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use task_options::TaskOptions;
use task_result::TaskResult;
use thepipelinetool_utils::{
    command_timeout, create_command, spawn, value_from_file, value_to_file,
};

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

// TODO move these
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
}

impl Task {
    pub fn execute<P, D>(
        &self,
        resolved_args: &Value,
        attempt: usize,
        handle_stdout_log: Box<dyn Fn(String) + Send>,
        handle_stderr_log: Box<dyn Fn(String) + Send>,
        take_last_stdout_line: Box<dyn Fn() -> String + Send>,
        dag_path: P,
        tpt_path: D,
        scheduled_date_for_dag_run: DateTime<Utc>,
        run_id: usize,
    ) -> TaskResult
    where
        P: AsRef<OsStr>,
        D: AsRef<OsStr>,
    {
        let task_id: usize = self.id;
        let function_name = &self.function;
        let resolved_args_str = serde_json::to_string(resolved_args).unwrap();
        let use_timeout = self.options.timeout.is_some();
        let timeout_as_secs = self
            .options
            .timeout
            .unwrap_or(Duration::ZERO)
            .as_secs()
            .to_string();

        let mut cmd = create_command(&dag_path, use_timeout, &tpt_path);
        cmd.env("run_id", run_id.to_string());
        command_timeout(
            &mut cmd,
            &dag_path,
            use_timeout,
            &timeout_as_secs,
            &tpt_path,
            &self.function,
        );

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
        let (status, timed_out) = spawn(cmd, handle_stdout_log, handle_stderr_log);
        let end = Utc::now();

        let (success, result) = (
            status.success(),
            match (status.success(), get_save_to_file()) {
                (true, true) => value_from_file(&out_path.unwrap()).unwrap(),
                (true, false) => serde_json::from_str(&take_last_stdout_line()).unwrap(),
                (false, _) => Value::Null,
            },
        );

        TaskResult {
            task_id,
            result,
            attempt,
            max_attempts: self.options.max_attempts,
            name: self.name.clone(),
            function: function_name.clone(),
            resolved_args_str,
            success,
            started: start.to_rfc3339(),
            ended: end.to_rfc3339(),
            elapsed: end.timestamp() - start.timestamp(),
            premature_failure: false,
            premature_failure_error_str: if timed_out { "timed out" } else { "" }.into(),
            is_branch: self.is_branch,
            is_sensor: self.options.is_sensor,
            scheduled_date_for_dag_run,
        }
    }
}
