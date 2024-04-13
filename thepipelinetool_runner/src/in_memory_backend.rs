use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Arc,
};

use crate::{backend::Run, Backend};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde_json::Value;
use thepipelinetool_task::{
    ordered_queued_task::OrderedQueuedTask, queued_task::QueuedTask, task_options::TaskOptions,
    task_result::TaskResult, task_status::TaskStatus, Task,
};

use anyhow::Result;

#[derive(Clone, Default)]
pub struct InMemoryBackend {
    pub task_results: Arc<Mutex<HashMap<usize, TaskResult>>>,
    pub task_logs: Arc<Mutex<HashMap<usize, Vec<String>>>>,
    pub task_statuses: Arc<Mutex<HashMap<usize, TaskStatus>>>,
    pub attempts: Arc<Mutex<HashMap<String, usize>>>,
    pub dependencies: Arc<Mutex<HashMap<usize, HashMap<(usize, String), String>>>>,
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
    fn get_task_depth(&mut self, run_id: usize, task_id: usize) -> Result<usize> {
        if !self.task_depth.lock().contains_key(&task_id) {
            let mut max_depth = 0;

            for upstream_id in self.get_upstream(run_id, task_id)? {
                let new_depth = self.get_task_depth(run_id, upstream_id)? + 1;
                if new_depth > max_depth {
                    max_depth = new_depth;
                }
            }
            // let depth = self
            //     .get_upstream(run_id, task_id)
            //     .iter()
            //     .map(|upstream_id: &usize| self.get_task_depth(run_id, *upstream_id) + 1)
            //     .max()
            //     .unwrap_or(0);
            self.task_depth.lock().insert(task_id, max_depth);
        }
        Ok(self.task_depth.lock()[&task_id])
    }

    fn set_task_depth(&mut self, _run_id: usize, task_id: usize, depth: usize) -> Result<()> {
        self.task_depth.lock().insert(task_id, depth);
        Ok(())
    }

    fn delete_task_depth(&mut self, _run_id: usize, task_id: usize) -> Result<()> {
        self.task_depth.lock().remove(&task_id);
        Ok(())
    }

    fn get_log(&mut self, _run_id: usize, task_id: usize, _attempt: usize) -> Result<String> {
        Ok(self
            .task_logs
            .lock()
            .get(&task_id)
            .unwrap_or(&vec![])
            .clone()
            .join("\n"))
    }

    fn get_log_handle_closure(
        &mut self,
        _run_id: usize,
        task_id: usize,
        _attempt: usize,
    ) -> Result<Box<dyn Fn(String) -> Result<()> + Send>> {
        let task_logs = self.task_logs.clone();
        Ok(Box::new(move |s| {
            task_logs.lock().entry(task_id).or_default().push(s);
            Ok(())
        }))
    }

    fn insert_task_results(&mut self, _run_id: usize, result: &TaskResult) -> Result<()> {
        // self.attempts.lock().insert(result.task_id, result.attempt);
        self.task_results
            .lock()
            .insert(result.task_id, result.clone());
        Ok(())
    }

    fn create_new_run(
        &mut self,
        // pipeline_name: &str,
        // _pipeline_hash: &str,
        scheduled_date_for_run: DateTime<Utc>,
    ) -> Result<Run> {
        Ok(Run {
            run_id: 0,
            pipeline_name: self.get_pipeline_name()?,
            scheduled_date_for_run,
        })
    }

    fn get_task_result(&mut self, _run_id: usize, task_id: usize) -> Result<TaskResult> {
        Ok(self.task_results.lock()[&task_id].clone())
    }

    fn get_attempt_by_task_id(
        &self,
        _run_id: usize,
        task_id: usize,
        is_dynamic: bool,
    ) -> Result<usize> {
        let key = format!("{task_id}{is_dynamic}");
        if !self.attempts.lock().contains_key(&key) {
            self.attempts.lock().insert(key.to_string(), 0);
        }
        let new_id = self.attempts.lock().get(&key).unwrap() + 1;
        self.attempts.lock().insert(key, new_id);
        Ok(new_id)
    }

    fn get_task_status(&self, _run_id: usize, task_id: usize) -> Result<TaskStatus> {
        Ok(match self.task_statuses.lock().get(&task_id) {
            Some(task_status) => task_status.clone(),
            None => TaskStatus::Pending,
        })
    }

    fn set_task_status(
        &mut self,
        _run_id: usize,
        task_id: usize,
        task_status: TaskStatus,
    ) -> Result<()> {
        self.task_statuses.lock().insert(task_id, task_status);
        Ok(())
    }

    fn get_dependencies(
        &mut self,
        _run_id: usize,
        task_id: usize,
    ) -> Result<HashMap<(usize, String), String>> {
        Ok(self.dependencies.lock().entry(task_id).or_default().clone())
    }

    fn get_downstream(&self, _run_id: usize, task_id: usize) -> Result<Vec<usize>> {
        let mut downstream: Vec<usize> = self
            .edges
            .lock()
            .iter()
            .filter(|(upstream_id, _)| upstream_id == &task_id)
            .map(|(_, downstream_id)| *downstream_id)
            .collect();
        downstream.sort();
        Ok(downstream)
    }

    fn remove_edge(&mut self, _run_id: usize, edge: (usize, usize)) -> Result<()> {
        let (upstream_id, downstream_id) = edge;
        self.dependencies
            .lock()
            .get_mut(&downstream_id)
            .unwrap_or(&mut HashMap::new())
            .remove(&(upstream_id, "".into()));

        self.edges.lock().remove(&edge);
        Ok(())
    }

    fn insert_edge(&mut self, _run_id: usize, edge: (usize, usize)) -> Result<()> {
        self.edges.lock().insert(edge);
        Ok(())
    }

    fn get_upstream(&self, _run_id: usize, task_id: usize) -> Result<Vec<usize>> {
        Ok(self
            .edges
            .lock()
            .iter()
            .filter(|(_, downstream)| downstream == &task_id)
            .map(|(upstream, _)| *upstream)
            .collect())
    }

    fn set_dependency(
        &mut self,
        _run_id: usize,
        task_id: usize,
        dependency: (usize, String),
        result_key: String,
    ) -> Result<()> {
        self.dependencies
            .lock()
            .entry(task_id)
            .or_default()
            .insert(dependency, result_key);
        Ok(())
    }

    fn get_default_tasks(&self) -> Result<Vec<Task>> {
        Ok(self.default_nodes.lock().clone())
    }

    fn get_default_edges(&self) -> Result<HashSet<(usize, usize)>> {
        Ok(self.edges.lock().clone())
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
    ) -> Result<usize> {
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
        Ok(new_id)
    }

    fn get_template_args(&self, _run_id: usize, task_id: usize) -> Result<Value> {
        Ok(self.nodes.lock()[task_id].template_args.clone())
    }

    fn set_template_args(
        &mut self,
        _run_id: usize,
        task_id: usize,
        template_args_str: &str,
    ) -> Result<()> {
        self.nodes.lock()[task_id].template_args = serde_json::from_str(template_args_str).unwrap();
        Ok(())
    }

    fn get_task_by_id(&self, _run_id: usize, task_id: usize) -> Result<Task> {
        Ok(self.nodes.lock()[task_id].clone())
    }

    fn get_all_tasks(&self, _run_id: usize) -> Result<Vec<Task>> {
        Ok(self.nodes.lock().clone())
    }

    fn print_priority_queue(&mut self) -> Result<()> {
        println!("{:#?}", self.priority_queue.lock());
        Ok(())
    }

    fn pop_priority_queue(&mut self) -> Result<Option<OrderedQueuedTask>> {
        let popped = self.priority_queue.lock().pop();
        if let Some(ordered_queued_task) = &popped {
            self.temp_queue
                .lock()
                .insert(ordered_queued_task.queued_task.clone());
        }
        Ok(popped)
    }

    fn enqueue_task(
        &mut self,
        run_id: usize,
        task_id: usize,
        scheduled_date_for_run: DateTime<Utc>,
        pipeline_name: String,
        is_dynamic: bool,
    ) -> Result<()> {
        let depth = self.get_task_depth(run_id, task_id)?;
        let mut priority_queue = self.priority_queue.lock();

        // remove previous attempts
        // priority_queue.retain(|x| x.queued_task.task_id != task_id);
        let attempt: usize = self.get_attempt_by_task_id(run_id, task_id, is_dynamic)?;

        priority_queue.push(OrderedQueuedTask {
            score: depth,
            queued_task: QueuedTask {
                task_id,
                run_id,
                pipeline_name,
                scheduled_date_for_run,
                attempt,
            },
        });
        Ok(())
    }

    fn take_last_stdout_line(
        &mut self,
        _run_id: usize,
        task_id: usize,
        _attempt: usize,
    ) -> Result<Box<dyn Fn() -> Result<String> + Send>> {
        let task_logs = self.task_logs.clone();
        Ok(Box::new(move || {
            Ok(task_logs.lock().entry(task_id).or_default().pop().unwrap())
        }))
    }

    fn get_queue_length(&self) -> Result<usize> {
        Ok(self.priority_queue.lock().len())
    }

    fn remove_from_temp_queue(&self, queued_task: &QueuedTask) -> Result<()> {
        self.temp_queue.lock().remove(queued_task);
        Ok(())
    }
    
    fn get_pipeline_name(&self) -> Result<String> {
        Ok("in_memory".into())
    }
}
