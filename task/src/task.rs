use std::{
    env,
    panic::RefUnwindSafe,
    process::{Command, Stdio},
    sync::{mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration, io::{BufReader, BufRead},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utils::{value_from_file, value_to_file};

use crate::{task_options::TaskOptions, task_result::TaskResult};

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: usize,
    // pub dag_name: String,
    pub function_name: String,
    pub template_args: Value,
    pub options: TaskOptions,
    pub lazy_expand: bool,
    pub is_dynamic: bool,
    pub is_branch: bool,
}

impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.depth.cmp(&self.depth)
            .then_with(|| other.task_id.cmp(&self.task_id))
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

    // fn clamp(self, min: Self, max: Self) -> Self
    // where
    //     Self: Sized,
    //     Self: PartialOrd,
    // {
    //     assert!(min <= max);
    //     if self < std::cmp::min {
    //         std::cmp::min
    //     } else if self > std::cmp::max {
    //         std::cmp::max
    //     } else {
    //         self
    //     }

    //     // todo!()
    // }
}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// impl PartialEq for QueuedTask {
//     fn eq(&self, other: &Self) -> bool {
//         self.depth == other.depth
//     }
// }

// impl Eq for QueuedTask {

// }

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct QueuedTask {
    pub depth: usize,
    pub task_id: usize,
    pub run_id: usize,
    pub dag_name: String,
}

impl QueuedTask {
    // pub fn new(depth: usize, task_id: usize) -> Self {
    //     Self {
    //         depth,
    //         task_id
    //     }
    // }

    pub fn increment(&self) -> Self {
        Self {
            depth: self.depth + 1,
            task_id: self.task_id,
            run_id: self.run_id,
            dag_name: self.dag_name.clone(),
        }
    }
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
        // tx: &Sender<(usize, TaskResult)>,
        handle_stdout: Box<dyn Fn(String) + Send>,
        handle_stderr: Box<dyn Fn(String) + Send>
    ) -> TaskResult {
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

        // let tx1 = tx.clone();

        // thread::spawn(move || {
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
            // .output()
            // .unwrap_or_else(|_| panic!("failed to run function: {}", function_name));

            let mut child = command.spawn().expect("failed to start command");

            let stdout = child.stdout.take().expect("failed to take stdout");
            let stderr = child.stderr.take().expect("failed to take stderr");

            let end = Utc::now();

            // let result_raw = String::from_utf8_lossy(&output.stdout);
            // let err_raw = String::from_utf8_lossy(&output.stderr);


            // Shared string for accumulating stdout
            let stdout_accum = Arc::new(Mutex::new(String::new()));

            // Clone the Arc to move into the stdout thread
            // let stdout_accum_clone = Arc::clone(&stdout_accum);


            // let handle = move |value: String| {
            //     // Deserialize the Value into T
            //     // let input: T = serde_json::from_value(value).unwrap();
            //     // // Call the original function
            //     // let output: G = function(input);
            //     // // Serialize the G type back into Value
            //     // serde_json::to_value(output).unwrap()
            //     handle_log(value);
            // };

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

            // let stderr_accum = Arc::new(Mutex::new(String::new()));

            // Clone the Arc to move into the stdout thread
            // let stderr_accum_clone = Arc::clone(&stderr_accum);


            // Spawn a thread to handle stderr
            let stderr_handle = thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    let line = format!("{}\n", line.expect("failed to read line from stdout"));
                    // println!("stderr: {}", &line);
                    // let mut accum = stderr_accum_clone.lock().unwrap();
                    // accum.push_str(&line);

                    handle_stderr(line);
                }
            });


            let status = child.wait().expect("failed to wait on child");
            let timed_out = matches!(status.code(), Some(124));

            // Join the stdout and stderr threads
            stdout_handle.join().expect("stdout thread panicked");
            stderr_handle.join().expect("stderr thread panicked");

            // let result_raw = stdout_accum.lock().unwrap().clone();
            // let err_raw = stderr_accum.lock().unwrap().clone();

            // tx1.send((
            //     dag_run_id,
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
                    // template_args_str,
                    resolved_args_str,
                    success: status.success(),
                    // stdout: result_raw.into(),
                    // stderr: err_raw.into(),
                    started: start.to_rfc3339(),
                    ended: end.to_rfc3339(),
                    elapsed: end.timestamp() - start.timestamp(),
                    premature_failure: false,
                    premature_failure_error_str: if timed_out {
                        "timed out".into()
                    } else {
                        "".into()
                    }
                    ,
                    is_branch,
                }
            // ))
            // .unwrap();
        // })
    }
}
