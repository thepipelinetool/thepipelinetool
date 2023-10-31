use std::{
    env,
    panic::RefUnwindSafe,
    process::Command,
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utils::{value_from_file, value_to_file};

use crate::{task_options::TaskOptions, task_result::TaskResult};

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

impl RefUnwindSafe for Task {}
pub const DAGS_DIR: &str = "./bin";
impl Task {
    pub fn execute(
        &self,
        dag_run_id: usize,
        dag_name: String,
        resolved_args: Value,
        attempt: usize,
        tx: &Sender<(usize, TaskResult)>,
    ) -> JoinHandle<()> {
        let task_id: usize = self.id;
        let function_name = self.function_name.clone();
        let template_args_str = serde_json::to_string_pretty(&self.template_args).unwrap();
        let resolved_args_str = serde_json::to_string_pretty(&resolved_args).unwrap();
        let retry_delay = self.options.retry_delay;
        let max_attempts = self.options.max_attempts;
        let timeout = self.options.timeout;
        let timeout_as_secs = timeout.unwrap_or(Duration::ZERO).as_secs().to_string();
        let is_branch = self.is_branch;

        let in_path = format!("./in_{function_name}_{task_id}.json");
        value_to_file(&resolved_args, &in_path);

        let tx1 = tx.clone();

        thread::spawn(move || {
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
            let mut binding = Command::new(f);
            let output = binding
                .args(a)
                .output()
                .unwrap_or_else(|_| panic!("failed to run function: {}", function_name));

            let end = Utc::now();

            let result_raw = String::from_utf8_lossy(&output.stdout);
            let err_raw = String::from_utf8_lossy(&output.stderr);

            let timed_out = matches!(output.status.code(), Some(124));

            tx1.send((
                dag_run_id,
                TaskResult {
                    task_id,
                    result: if output.status.success() {
                        value_from_file(&out_path)
                    } else {
                        Value::Null
                    },
                    attempt,
                    max_attempts,
                    function_name,
                    template_args_str,
                    resolved_args_str,
                    success: output.status.success(),
                    stdout: result_raw.into(),
                    stderr: err_raw.into(),
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
                },
            ))
            .unwrap();
        })
    }
}
