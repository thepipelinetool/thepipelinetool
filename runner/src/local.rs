use std::{
    collections::{hash_map::DefaultHasher, BinaryHeap, HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex, RwLock},
};

use chrono::{DateTime, Utc};
use serde_json::Value;
use task::{
    task::{QueuedTask, Task},
    task_result::TaskResult,
    task_status::TaskStatus,
};

use crate::{DefRunner, Runner};

pub struct LocalRunner {
    task_results: HashMap<usize, TaskResult>,
    task_logs: Arc<Mutex<HashMap<usize, String>>>,
    task_statuses: HashMap<usize, TaskStatus>,
    attempts: HashMap<usize, usize>,
    dep_keys: HashMap<usize, HashMap<(usize, Option<String>), Option<String>>>,
    edges: HashSet<(usize, usize)>,
    default_nodes: Vec<Task>,
    nodes: Vec<Task>,
    priority_queue: BinaryHeap<QueuedTask>,
    task_depth: HashMap<usize, usize>,
}

impl LocalRunner {
    pub fn new(_name: &str, nodes: &[Task], edges: &HashSet<(usize, usize)>) -> Self {
        Self {
            default_nodes: nodes.to_vec(),
            nodes: Vec::new(),
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

impl Runner for LocalRunner {
    fn get_task_depth(&mut self, _dag_run_id: &usize, task_id: &usize) -> usize {
        if !self.task_depth.contains_key(task_id) {
            let depth = self
                .get_upstream(_dag_run_id, task_id)
                .iter()
                .map(|up| self.get_task_depth(_dag_run_id, up) + 1)
                .max()
                .unwrap_or(0);
            self.task_depth.insert(*task_id, depth);
        }
        self.task_depth[task_id]
    }

    fn set_task_depth(&mut self, _dag_run_id: &usize, task_id: &usize, depth: usize) {
        self.task_depth.insert(*task_id, depth);
    }

    fn get_log(
        &mut self,
        _dag_run_id: &usize,
        task_id: &usize,
        _attempt: usize,
        // pool: Pool<Postgres>,
    ) -> String {
        dbg!(task_id);
        self.task_logs
            .lock()
            .unwrap()
            .get(task_id)
            .unwrap_or(&"".to_string())
            .clone()
    }

    // fn init_log(&mut self, _dag_run_id: &usize, task_id: &usize, _attempt: usize) {
    //     dbg!(&task_id);
    //     let mut task_logs = self.task_logs.lock().unwrap();
    //     task_logs.insert(*task_id, "".into());
    // }

    fn handle_log(
        &mut self,
        _dag_run_id: &usize,
        task_id: &usize,
        _attempt: usize,
    ) -> Box<dyn Fn(String) + Send> {
        let task_logs = self.task_logs.clone();
        let task_id = *task_id;

        Box::new(move |s| {
            let mut task_logs = task_logs.lock().unwrap();
            if !task_logs.contains_key(&task_id) {
                task_logs.insert(task_id, "".into());
            }
            *task_logs.get_mut(&task_id).unwrap() += &s;
        })
    }

    fn insert_task_results(&mut self, _dag_run_id: &usize, result: &TaskResult) {
        // dbg!(&self.task_logs.lock().unwrap());

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

    fn get_task_result(&mut self, _dag_run_id: &usize, task_id: &usize) -> TaskResult {
        self.task_results[task_id].clone()
    }

    fn get_attempt_by_task_id(&self, _dag_run_id: &usize, task_id: &usize) -> usize {
        if !self.attempts.contains_key(task_id) {
            return 1;
        }
        *self.attempts.get(task_id).unwrap() + 1
    }

    fn get_task_status(&mut self, _dag_run_id: &usize, task_id: &usize) -> TaskStatus {
        match self.task_statuses.get(task_id) {
            Some(task_status) => task_status.clone(),
            None => TaskStatus::Pending,
        }
    }

    fn set_task_status(&mut self, _dag_run_id: &usize, task_id: &usize, task_status: TaskStatus) {
        self.task_statuses.insert(*task_id, task_status);
    }

    fn mark_finished(&self, _dag_run_id: &usize) {
        // todo!()
    }

    fn any_upstream_incomplete(&mut self, dag_run_id: &usize, task_id: &usize) -> bool {
        self.get_upstream(dag_run_id, task_id)
            .iter()
            .any(|edge| !self.is_task_completed(dag_run_id, edge))
    }

    fn get_dependency_keys(
        &self,
        _dag_run_id: &usize,
        task_id: &usize,
    ) -> HashMap<(usize, Option<String>), Option<String>> {
        let h: HashMap<(usize, Option<String>), Option<String>> = HashMap::new();

        self.dep_keys.get(task_id).unwrap_or(&h).clone()
    }

    fn get_downstream(&self, _dag_run_id: &usize, task_id: &usize) -> HashSet<usize> {
        HashSet::from_iter(
            self.edges
                .iter()
                .filter(|(u, _)| u == task_id)
                .map(|(_, d)| *d),
        )
    }

    fn remove_edge(&mut self, _dag_run_id: &usize, edge: &(usize, usize)) {
        let (upstream_task_id, downstream_task_id) = *edge;
        self.dep_keys
            .get_mut(&downstream_task_id)
            .unwrap_or(&mut HashMap::new())
            .remove(&(upstream_task_id, None));

        self.edges.remove(edge);
    }

    fn insert_edge(&mut self, _dag_run_id: &usize, edge: &(usize, usize)) {
        self.edges.insert(*edge);
    }

    fn get_upstream(&self, _dag_run_id: &usize, task_id: &usize) -> HashSet<usize> {
        HashSet::from_iter(
            self.edges
                .iter()
                .filter(|(_, d)| d == task_id)
                .map(|(u, _)| *u),
        )
    }

    fn set_dependency_keys(
        &mut self,
        _dag_run_id: &usize,
        task_id: &usize,
        upstream: (usize, Option<String>),
        v: Option<String>,
    ) {
        self.dep_keys
            .entry(*task_id)
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
        _dag_run_id: &usize,
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
            // name,
            function_name,
            template_args,
            options,
            lazy_expand,
            is_dynamic,
            is_branch,
        });
        new_id
    }

    fn get_template_args(&self, _dag_run_id: &usize, task_id: &usize) -> Value {
        self.nodes[*task_id].template_args.clone()
    }

    fn set_template_args(&mut self, _dag_run_id: &usize, task_id: &usize, template_args_str: &str) {
        self.nodes[*task_id].template_args = serde_json::from_str(template_args_str).unwrap();
    }

    fn get_task_by_id(&self, _dag_run_id: &usize, task_id: &usize) -> Task {
        self.nodes[*task_id].clone()
    }

    // fn set_status_to_running_if_possible(&mut self, dag_run_id: &usize, task_id: &usize) -> bool {
    //     let current_task_status = self.get_task_status(dag_run_id, task_id);

    //     if !(current_task_status == TaskStatus::Pending
    //         || current_task_status == TaskStatus::Retrying)
    //     {
    //         return false;
    //     }
    //     self.set_task_status(dag_run_id, task_id, TaskStatus::Running);
    //     true
    // }

    fn get_dag_name(&self) -> String {
        "default".into()
    }

    fn get_all_tasks_incomplete(&mut self, dag_run_id: &usize) -> Vec<Task> {
        self.nodes
            .clone()
            .iter()
            .filter(|n| !self.is_task_completed(dag_run_id, &n.id))
            .cloned()
            // .map(|t| t.clone())
            .collect()
    }

    fn get_all_tasks(&self, _dag_run_id: &usize) -> Vec<Task> {
        self.nodes.clone()
    }

    fn enqueue_task(&mut self, dag_run_id: &usize, task_id: &usize) {
        let depth = self.get_task_depth(dag_run_id, task_id);
        self.priority_queue.push(QueuedTask::new(depth, *task_id));
    }

    fn print_priority_queue(&mut self) {
        // self.priority_queue.iter().collect()
        // let mut vec = vec![];
        // let len = self.priority_queue.len();
        // for i in 0..len {
        //     vec.push(self.priority_queue.pop().unwrap());
        // }
        // vec
        println!("{:#?}", self.priority_queue);
        // println!("{:?}", &self.task_statuses);
    }

    fn pop_priority_queue(&mut self) -> Option<QueuedTask> {
        self.priority_queue.pop()
    }
    fn push_priority_queue(&mut self, queued_task: QueuedTask) {
        self.priority_queue.push(queued_task);
    }

    fn priority_queue_len(&self) -> usize {
        self.priority_queue.len()
    }
}
