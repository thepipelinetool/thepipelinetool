use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use serde_json::Value;
use task::{
    task::{OrderedQueuedTask, QueuedTask, Task},
    task_result::TaskResult,
    task_status::TaskStatus,
};

use crate::{DefRunner, Runner};

pub struct InMemoryRunner {
    task_results: HashMap<usize, TaskResult>,
    task_logs: Arc<Mutex<HashMap<usize, String>>>,
    task_statuses: HashMap<usize, TaskStatus>,
    attempts: HashMap<usize, usize>,
    dep_keys: HashMap<usize, HashMap<(usize, String), String>>,
    edges: HashSet<(usize, usize)>,
    default_nodes: Vec<Task>,
    nodes: Vec<Task>,
    priority_queue: BinaryHeap<OrderedQueuedTask>,
    task_depth: HashMap<usize, usize>,
}

impl InMemoryRunner {
    pub fn new(_name: &str, nodes: &[Task], edges: &HashSet<(usize, usize)>) -> Self {
        Self {
            default_nodes: nodes.to_vec(),
            nodes: vec![],
            edges: edges.clone(),
            task_results: HashMap::new(),
            task_statuses: HashMap::new(),
            attempts: HashMap::new(),
            dep_keys: HashMap::new(),
            task_logs: Arc::new(Mutex::new(HashMap::new())),
            priority_queue: BinaryHeap::new(),
            task_depth: HashMap::new(),
        }
    }
}

impl Runner for InMemoryRunner {
    fn get_task_depth(&mut self, run_id: usize, task_id: usize) -> usize {
        if !self.task_depth.contains_key(&task_id) {
            let depth = self
                .get_upstream(run_id, task_id)
                .iter()
                .map(|upstream_id: &usize| self.get_task_depth(run_id, *upstream_id) + 1)
                .max()
                .unwrap_or(0);
            self.task_depth.insert(task_id, depth);
        }
        self.task_depth[&task_id]
    }

    fn set_task_depth(&mut self, _run_id: usize, task_id: usize, depth: usize) {
        self.task_depth.insert(task_id, depth);
    }

    fn delete_task_depth(&mut self, _run_id: usize, task_id: usize) {
        self.task_depth.remove(&task_id);
    }

    fn get_log(&mut self, _run_id: usize, task_id: usize, _attempt: usize) -> String {
        self.task_logs
            .lock()
            .unwrap()
            .get(&task_id)
            .unwrap_or(&"".to_string())
            .clone()
    }

    fn handle_log(
        &mut self,
        _run_id: usize,
        task_id: usize,
        _attempt: usize,
    ) -> Box<dyn Fn(String) + Send> {
        let task_logs = self.task_logs.clone();
        Box::new(move |s| {
            let mut task_logs = task_logs.lock().unwrap();
            let log = task_logs.entry(task_id).or_insert_with(|| "".into());
            *log += &s;
        })
    }

    fn insert_task_results(&mut self, _run_id: usize, result: &TaskResult) {
        self.attempts.insert(result.task_id, result.attempt);
        self.task_results.insert(result.task_id, result.clone());
    }

    fn create_new_run(
        &mut self,
        _dag_name: &str,
        _dag_hash: &str,
        _logical_date: DateTime<Utc>,
    ) -> usize {
        0
    }

    fn get_task_result(&mut self, _run_id: usize, task_id: usize) -> TaskResult {
        self.task_results[&task_id].clone()
    }

    fn get_attempt_by_task_id(&self, _run_id: usize, task_id: usize) -> usize {
        if !self.attempts.contains_key(&task_id) {
            return 1;
        }
        *self.attempts.get(&task_id).unwrap() + 1
    }

    fn get_task_status(&mut self, _run_id: usize, task_id: usize) -> TaskStatus {
        match self.task_statuses.get(&task_id) {
            Some(task_status) => task_status.clone(),
            None => TaskStatus::Pending,
        }
    }

    fn set_task_status(&mut self, _run_id: usize, task_id: usize, task_status: TaskStatus) {
        self.task_statuses.insert(task_id, task_status);
    }

    fn any_upstream_incomplete(&mut self, run_id: usize, task_id: usize) -> bool {
        self.get_upstream(run_id, task_id)
            .iter()
            .any(|edge| !self.is_task_completed(run_id, *edge))
    }

    fn get_dependency_keys(
        &mut self,
        _run_id: usize,
        task_id: usize,
    ) -> HashMap<(usize, String), String> {
        self.dep_keys.entry(task_id).or_default().clone()
    }

    fn get_downstream(&self, _run_id: usize, task_id: usize) -> Vec<usize> {
        let mut downstream: Vec<usize> = self
            .edges
            .iter()
            .filter(|(upstream_id, _)| upstream_id == &task_id)
            .map(|(_, downstream_id)| *downstream_id)
            .collect();
        downstream.sort();
        downstream
    }

    fn remove_edge(&mut self, _run_id: usize, edge: (usize, usize)) {
        let (upstream_task_id, downstream_task_id) = edge;
        self.dep_keys
            .get_mut(&downstream_task_id)
            .unwrap_or(&mut HashMap::new())
            .remove(&(upstream_task_id, "".into()));

        self.edges.remove(&edge);
    }

    fn insert_edge(&mut self, _run_id: usize, edge: (usize, usize)) {
        self.edges.insert(edge);
    }

    fn get_upstream(&self, _run_id: usize, task_id: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|(_, d)| d == &task_id)
            .map(|(u, _)| *u)
            .collect()
    }

    fn set_dependency_keys(
        &mut self,
        _run_id: usize,
        task_id: usize,
        upstream: (usize, String),
        v: String,
    ) {
        self.dep_keys
            .entry(task_id)
            .or_default()
            .insert(upstream, v);
    }

    fn get_default_tasks(&self) -> Vec<Task> {
        self.default_nodes.clone()
    }

    fn get_default_edges(&self) -> HashSet<(usize, usize)> {
        self.edges.clone()
    }

    fn append_new_task_and_set_status_to_pending(
        &mut self,
        _run_id: usize,
        function_name: String,
        template_args: Value,
        options: task::task_options::TaskOptions,
        lazy_expand: bool,
        is_dynamic: bool,
        is_branch: bool,
    ) -> usize {
        let new_id = self.nodes.len();
        self.nodes.push(Task {
            id: new_id,
            function_name,
            template_args,
            options,
            lazy_expand,
            is_dynamic,
            is_branch,
        });
        new_id
    }

    fn get_template_args(&self, _run_id: usize, task_id: usize) -> Value {
        self.nodes[task_id].template_args.clone()
    }

    fn set_template_args(&mut self, _run_id: usize, task_id: usize, template_args_str: &str) {
        self.nodes[task_id].template_args = serde_json::from_str(template_args_str).unwrap();
    }

    fn get_task_by_id(&self, _run_id: usize, task_id: usize) -> Task {
        self.nodes[task_id].clone()
    }

    fn get_dag_name(&self) -> String {
        "in_memory".into()
    }

    fn get_all_tasks(&self, _run_id: usize) -> Vec<Task> {
        self.nodes.clone()
    }

    fn enqueue_task(&mut self, run_id: usize, task_id: usize) {
        let depth = self.get_task_depth(run_id, task_id);
        self.priority_queue
            .retain(|x| x.queued_task.task_id != task_id);
        self.priority_queue.push(OrderedQueuedTask {
            score: depth,
            queued_task: QueuedTask {
                task_id,
                run_id,
                dag_name: self.get_dag_name(),
            },
        });
    }

    fn print_priority_queue(&mut self) {
        println!("{:#?}", self.priority_queue);
    }

    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask> {
        self.priority_queue.pop()
    }
    fn push_priority_queue(&mut self, queued_task: OrderedQueuedTask) {
        self.priority_queue.push(queued_task);
    }
}
