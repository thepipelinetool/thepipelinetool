use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Arc,
};

use crate::Backend;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde_json::Value;
use thepipelinetool_task::{
    ordered_queued_task::OrderedQueuedTask, queued_task::QueuedTask, task_options::TaskOptions,
    task_result::TaskResult, task_status::TaskStatus, Task,
};

#[derive(Clone, Default)]
pub struct InMemoryBackend {
    pub task_results: Arc<Mutex<HashMap<usize, TaskResult>>>,
    pub task_logs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
    pub task_statuses: Arc<Mutex<HashMap<usize, TaskStatus>>>,
    pub attempts: Arc<Mutex<HashMap<String, usize>>>,
    pub dep_keys: Arc<Mutex<HashMap<usize, HashMap<(usize, String), String>>>>,
    pub edges: Arc<Mutex<HashSet<(usize, usize)>>>,
    pub default_nodes: Arc<Mutex<Vec<Task>>>,
    pub nodes: Arc<Mutex<Vec<Task>>>,
    pub task_depth: Arc<Mutex<HashMap<usize, usize>>>,
    pub priority_queue: Arc<Mutex<BinaryHeap<OrderedQueuedTask>>>,
    pub temp_queue: Arc<Mutex<HashSet<QueuedTask>>>,
}

impl InMemoryBackend {
    pub fn new(nodes: &[Task], edges: &HashSet<(usize, usize)>) -> Self {
        Self {
            edges: Arc::new(Mutex::new(edges.clone())),
            default_nodes: Arc::new(Mutex::new(nodes.to_vec())),
            ..Default::default()
        }
    }
}

impl Backend for InMemoryBackend {
    fn get_task_depth(&mut self, run_id: usize, task_id: usize) -> usize {
        if !self.task_depth.lock().contains_key(&task_id) {
            let depth = self
                .get_upstream(run_id, task_id)
                .iter()
                .map(|upstream_id: &usize| self.get_task_depth(run_id, *upstream_id) + 1)
                .max()
                .unwrap_or(0);
            self.task_depth.lock().insert(task_id, depth);
        }
        self.task_depth.lock()[&task_id]
    }

    fn set_task_depth(&mut self, _run_id: usize, task_id: usize, depth: usize) {
        self.task_depth.lock().insert(task_id, depth);
    }

    fn delete_task_depth(&mut self, _run_id: usize, task_id: usize) {
        self.task_depth.lock().remove(&task_id);
    }

    fn get_log(&mut self, _run_id: usize, task_id: usize, _attempt: usize) -> String {
        self.task_logs
            .lock()
            .get(&task_id)
            .unwrap_or(&vec![])
            .clone()
            .join("\n")
    }

    fn get_log_handle_closure(
        &mut self,
        _run_id: usize,
        task_id: usize,
        _attempt: usize,
    ) -> Box<dyn Fn(String) + Send> {
        let task_logs = self.task_logs.clone();
        Box::new(move |s| task_logs.lock().entry(task_id).or_default().push(s))
    }

    fn insert_task_results(&mut self, _run_id: usize, result: &TaskResult) {
        // self.attempts.lock().insert(result.task_id, result.attempt);
        self.task_results
            .lock()
            .insert(result.task_id, result.clone());
    }

    fn create_new_run(
        &mut self,
        _dag_name: &str,
        _dag_hash: &str,
        _scheduled_date_for_dag_run: DateTime<Utc>,
    ) -> usize {
        0
    }

    fn get_task_result(&mut self, _run_id: usize, task_id: usize) -> TaskResult {
        self.task_results.lock()[&task_id].clone()
    }

    fn get_attempt_by_task_id(&self, _run_id: usize, task_id: usize, is_dynamic: bool) -> usize {
        let key = format!("{task_id}{is_dynamic}");
        if !self.attempts.lock().contains_key(&key) {
            self.attempts.lock().insert(key.to_string(), 0);
        }
        let new_id = self.attempts.lock().get(&key).unwrap() + 1;
        self.attempts.lock().insert(key, new_id);
        new_id
    }

    fn get_task_status(&self, _run_id: usize, task_id: usize) -> TaskStatus {
        match self.task_statuses.lock().get(&task_id) {
            Some(task_status) => task_status.clone(),
            None => TaskStatus::Pending,
        }
    }

    fn set_task_status(&mut self, _run_id: usize, task_id: usize, task_status: TaskStatus) {
        self.task_statuses.lock().insert(task_id, task_status);
    }

    fn get_dependency_keys(
        &mut self,
        _run_id: usize,
        task_id: usize,
    ) -> HashMap<(usize, String), String> {
        self.dep_keys.lock().entry(task_id).or_default().clone()
    }

    fn get_downstream(&self, _run_id: usize, task_id: usize) -> Vec<usize> {
        let mut downstream: Vec<usize> = self
            .edges
            .lock()
            .iter()
            .filter(|(upstream_id, _)| upstream_id == &task_id)
            .map(|(_, downstream_id)| *downstream_id)
            .collect();
        downstream.sort();
        downstream
    }

    fn remove_edge(&mut self, _run_id: usize, edge: (usize, usize)) {
        let (upstream_id, downstream_id) = edge;
        self.dep_keys
            .lock()
            .get_mut(&downstream_id)
            .unwrap_or(&mut HashMap::new())
            .remove(&(upstream_id, "".into()));

        self.edges.lock().remove(&edge);
    }

    fn insert_edge(&mut self, _run_id: usize, edge: (usize, usize)) {
        self.edges.lock().insert(edge);
    }

    fn get_upstream(&self, _run_id: usize, task_id: usize) -> Vec<usize> {
        self.edges
            .lock()
            .iter()
            .filter(|(_, downstream)| downstream == &task_id)
            .map(|(upstream, _)| *upstream)
            .collect()
    }

    fn set_dependency_keys(
        &mut self,
        _run_id: usize,
        task_id: usize,
        upstream: (usize, String),
        result_key: String,
    ) {
        self.dep_keys
            .lock()
            .entry(task_id)
            .or_default()
            .insert(upstream, result_key);
    }

    fn get_default_tasks(&self) -> Vec<Task> {
        self.default_nodes.lock().clone()
    }

    fn get_default_edges(&self) -> HashSet<(usize, usize)> {
        self.edges.lock().clone()
    }

    fn append_new_task_and_set_status_to_pending(
        &mut self,
        _run_id: usize,
        name: &str,
        function_name: &str,
        template_args: &Value,
        options: &TaskOptions,
        lazy_expand: bool,
        is_dynamic: bool,
        is_branch: bool,
        use_trigger_params: bool,
    ) -> usize {
        let mut nodes = self.nodes.lock();
        let new_id = nodes.len();
        nodes.push(Task {
            id: new_id,
            name: name.to_owned(),
            function: function_name.to_owned(),
            template_args: template_args.to_owned(),
            options: options.to_owned(),
            lazy_expand,
            is_dynamic,
            is_branch,
            use_trigger_params,
        });
        new_id
    }

    fn get_template_args(&self, _run_id: usize, task_id: usize) -> Value {
        self.nodes.lock()[task_id].template_args.clone()
    }

    fn set_template_args(&mut self, _run_id: usize, task_id: usize, template_args_str: &str) {
        self.nodes.lock()[task_id].template_args = serde_json::from_str(template_args_str).unwrap();
    }

    fn get_task_by_id(&self, _run_id: usize, task_id: usize) -> Task {
        self.nodes.lock()[task_id].clone()
    }

    fn get_all_tasks(&self, _run_id: usize) -> Vec<Task> {
        self.nodes.lock().clone()
    }

    fn print_priority_queue(&mut self) {
        println!("{:#?}", self.priority_queue.lock());
    }

    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask> {
        let popped = self.priority_queue.lock().pop();
        if let Some(ordered_queued_task) = &popped {
            self.temp_queue
                .lock()
                .insert(ordered_queued_task.queued_task.clone());
        }
        popped
    }

    fn enqueue_task(
        &mut self,
        run_id: usize,
        task_id: usize,
        scheduled_date_for_dag_run: DateTime<Utc>,
        dag_name: String,
        is_dynamic: bool,
    ) {
        let depth = self.get_task_depth(run_id, task_id);
        let mut priority_queue = self.priority_queue.lock();

        // remove previous attempts
        // priority_queue.retain(|x| x.queued_task.task_id != task_id);
        let attempt: usize = self.get_attempt_by_task_id(run_id, task_id, is_dynamic);

        priority_queue.push(OrderedQueuedTask {
            score: depth,
            queued_task: QueuedTask {
                task_id,
                run_id,
                dag_name,
                scheduled_date_for_dag_run,
                attempt,
            },
        });
    }

    fn take_last_stdout_line(
        &mut self,
        _run_id: usize,
        task_id: usize,
        _attempt: usize,
    ) -> Box<dyn Fn() -> String + Send> {
        let task_logs = self.task_logs.clone();
        Box::new(move || task_logs.lock().entry(task_id).or_default().pop().unwrap())
    }

    fn get_queue_length(&self) -> usize {
        self.priority_queue.lock().len()
    }

    fn remove_from_temp_queue(&self, queued_task: &QueuedTask) {
        self.temp_queue.lock().remove(queued_task);
    }
}
