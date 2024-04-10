use std::collections::{HashMap, HashSet};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use thepipelinetool_task::{
    ordered_queued_task::OrderedQueuedTask, queued_task::QueuedTask, task_options::TaskOptions,
    task_result::TaskResult, task_status::TaskStatus, Task,
};

pub trait Backend {
    // fn load_from_name(&mut self, dag_name: &str);

    fn remove_from_temp_queue(&self, queued_task: &QueuedTask) -> Result<()>;
    fn get_queue_length(&self) -> Result<usize>;

    fn print_priority_queue(&mut self) -> Result<()>;
    fn pop_priority_queue(&mut self) -> Result<Option<OrderedQueuedTask>>;
    fn enqueue_task(
        &mut self,
        run_id: usize,
        task_id: usize,
        scheduled_date_for_dag_run: DateTime<Utc>,
        dag_name: String,
        is_dynamic: bool,
    ) -> Result<()>;

    // fn get_dag_name(&self) -> String;
    fn get_log(&mut self, run_id: usize, task_id: usize, attempt: usize) -> Result<String>;
    fn get_log_handle_closure(
        &mut self,
        run_id: usize,
        task_id: usize,
        attempt: usize,
    ) -> Result<Box<dyn Fn(String) -> Result<()> + Send>>;
    fn take_last_stdout_line(
        &mut self,
        run_id: usize,
        task_id: usize,
        attempt: usize,
    ) -> Result<Box<dyn Fn() -> Result<String> + Send>>;

    fn get_task_result(&mut self, run_id: usize, task_id: usize) -> Result<TaskResult>;
    fn insert_task_results(&mut self, run_id: usize, result: &TaskResult) -> Result<()>;

    fn get_task_status(&self, run_id: usize, task_id: usize) -> Result<TaskStatus>;
    fn set_task_status(
        &mut self,
        run_id: usize,
        task_id: usize,
        task_status: TaskStatus,
    ) -> Result<()>;

    fn get_downstream(&self, run_id: usize, task_id: usize) -> Result<Vec<usize>>;
    fn get_upstream(&self, run_id: usize, task_id: usize) -> Result<Vec<usize>>;

    fn get_default_tasks(&self) -> Result<Vec<Task>>;
    fn get_all_tasks(&self, run_id: usize) -> Result<Vec<Task>>;
    fn get_default_edges(&self) -> Result<HashSet<(usize, usize)>>;
    fn get_task_by_id(&self, run_id: usize, task_id: usize) -> Result<Task>;
    fn get_template_args(&self, run_id: usize, task_id: usize) -> Result<Value>;

    fn set_template_args(
        &mut self,
        run_id: usize,
        task_id: usize,
        template_args_str: &str,
    ) -> Result<()>;

    fn get_task_depth(&mut self, run_id: usize, task_id: usize) -> Result<usize>;
    fn get_dependency_keys(
        &mut self,
        run_id: usize,
        task_id: usize,
    ) -> Result<HashMap<(usize, String), String>>;
    fn set_dependency_keys(
        &mut self,
        run_id: usize,
        task_id: usize,
        upstream: (usize, String),
        v: String,
    ) -> Result<()>;
    fn set_task_depth(&mut self, run_id: usize, task_id: usize, depth: usize) -> Result<()>;
    fn delete_task_depth(&mut self, run_id: usize, task_id: usize) -> Result<()>;

    fn get_attempt_by_task_id(
        &self,
        run_id: usize,
        task_id: usize,
        is_dynamic: bool,
    ) -> Result<usize>;

    fn create_new_run(
        &mut self,
        dag_name: &str,
        dag_hash: &str,
        scheduled_date_for_dag_run: DateTime<Utc>,
    ) -> Result<usize>;

    fn remove_edge(&mut self, run_id: usize, edge: (usize, usize)) -> Result<()>;
    fn insert_edge(&mut self, run_id: usize, edge: (usize, usize)) -> Result<()>;

    fn append_new_task_and_set_status_to_pending(
        &mut self,
        run_id: usize,
        name: &str,
        function_name: &str,
        template_args: &Value,
        options: &TaskOptions,
        lazy_expand: bool,
        is_dynamic: bool,
        is_branch: bool,
        use_trigger_params: bool,
    ) -> Result<usize>;
}
