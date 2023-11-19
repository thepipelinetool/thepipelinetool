use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde_json::Value;
use task::{
    ordered_queued_task::OrderedQueuedTask, task_options::TaskOptions, task_result::TaskResult,
    task_status::TaskStatus, Task,
};

pub mod blanket;
pub mod in_memory;

pub trait Runner {
    fn get_dag_name(&self) -> String;

    fn print_priority_queue(&mut self);
    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask>;
    fn push_priority_queue(&mut self, queued_task: OrderedQueuedTask);

    fn get_log(&mut self, run_id: usize, task_id: usize, attempt: usize) -> String;
    fn get_log_handle_closure(
        &mut self,
        run_id: usize,
        task_id: usize,
        attempt: usize,
    ) -> Box<dyn Fn(String) + Send>;

    fn get_task_result(&mut self, run_id: usize, task_id: usize) -> TaskResult;
    fn insert_task_results(&mut self, run_id: usize, result: &TaskResult);

    fn get_task_status(&mut self, run_id: usize, task_id: usize) -> TaskStatus;
    fn set_task_status(&mut self, run_id: usize, task_id: usize, task_status: TaskStatus);

    fn get_downstream(&self, run_id: usize, task_id: usize) -> Vec<usize>;
    fn get_upstream(&self, run_id: usize, task_id: usize) -> Vec<usize>;
    fn any_upstream_incomplete(&mut self, run_id: usize, task_id: usize) -> bool;

    fn get_default_tasks(&self) -> Vec<Task>;
    fn get_all_tasks(&self, run_id: usize) -> Vec<Task>;
    fn get_default_edges(&self) -> HashSet<(usize, usize)>;
    fn get_task_by_id(&self, run_id: usize, task_id: usize) -> Task;
    fn get_template_args(&self, run_id: usize, task_id: usize) -> Value;

    fn set_template_args(&mut self, run_id: usize, task_id: usize, template_args_str: &str);

    fn get_task_depth(&mut self, run_id: usize, task_id: usize) -> usize;
    fn get_dependency_keys(
        &mut self,
        run_id: usize,
        task_id: usize,
    ) -> HashMap<(usize, String), String>;
    fn set_dependency_keys(
        &mut self,
        run_id: usize,
        task_id: usize,
        upstream: (usize, String),
        v: String,
    );
    fn set_task_depth(&mut self, run_id: usize, task_id: usize, depth: usize);
    fn delete_task_depth(&mut self, run_id: usize, task_id: usize);

    fn get_attempt_by_task_id(&self, run_id: usize, task_id: usize) -> usize;

    fn create_new_run(
        &mut self,
        dag_name: &str,
        dag_hash: &str,
        logical_date: DateTime<Utc>,
    ) -> usize;

    fn remove_edge(&mut self, run_id: usize, edge: (usize, usize));
    fn insert_edge(&mut self, run_id: usize, edge: (usize, usize));

    fn append_new_task_and_set_status_to_pending(
        &mut self,
        run_id: usize,
        function_name: &str,
        template_args: &Value,
        options: &TaskOptions,
        lazy_expand: bool,
        is_dynamic: bool,
        is_branch: bool,
    ) -> usize;

    fn enqueue_task(&mut self, run_id: usize, task_id: usize);
}
