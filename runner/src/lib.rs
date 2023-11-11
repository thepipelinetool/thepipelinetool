use std::{
    collections::{HashMap, HashSet},
    io::{Error, ErrorKind},
    sync::mpsc::{self, Sender},
};

use chrono::{DateTime, Utc};
use local::hash_dag;
use serde_json::{json, Value};
use task::{
    task::Task, task_options::TaskOptions, task_ref::TaskRef, task_result::TaskResult,
    task_status::TaskStatus,
};
use utils::function_name_as_string;

pub mod local;

pub trait Runner {
    fn get_dag_name(&self) -> String;
    fn set_status_to_running_if_possible(&mut self, dag_run_id: &usize, task_id: &usize) -> bool;
    fn get_task_result(&mut self, dag_run_id: &usize, task_id: &usize) -> TaskResult;
    fn get_attempt_by_task_id(&self, dag_run_id: &usize, task_id: &usize) -> usize;
    fn get_task_status(&mut self, dag_run_id: &usize, task_id: &usize) -> TaskStatus;
    fn set_task_status(&mut self, dag_run_id: &usize, task_id: &usize, task_status: TaskStatus);
    fn create_new_run(
        &mut self,
        dag_name: &str,
        dag_hash: &str,
        logical_date: DateTime<Utc>,
    ) -> usize;
    fn insert_task_results(&mut self, dag_run_id: &usize, result: &TaskResult);
    fn mark_finished(&self, dag_run_id: &usize);
    fn any_upstream_incomplete(&mut self, dag_run_id: &usize, task_id: &usize) -> bool;
    fn get_dependency_keys(
        &self,
        dag_run_id: &usize,
        task_id: &usize,
    ) -> HashMap<(usize, Option<String>), Option<String>>;
    fn set_dependency_keys(
        &mut self,
        dag_run_id: &usize,
        task_id: &usize,
        upstream: (usize, Option<String>),
        v: Option<String>,
    );
    fn get_downstream(&self, dag_run_id: &usize, task_id: &usize) -> HashSet<usize>;
    fn get_upstream(&self, dag_run_id: &usize, task_id: &usize) -> HashSet<usize>;
    fn remove_edge(&mut self, dag_run_id: &usize, edge: &(usize, usize));
    fn insert_edge(&mut self, dag_run_id: &usize, edge: &(usize, usize));
    fn get_default_tasks(&self) -> Vec<Task>;
    fn get_all_tasks_incomplete(&mut self, dag_run_id: &usize) -> Vec<Task>;
    fn get_all_tasks(&self, dag_run_id: &usize) -> Vec<Task>;

    fn get_default_edges(&self) -> HashSet<(usize, usize)>;
    fn get_task_by_id(&self, dag_run_id: &usize, task_id: &usize) -> Task;
    fn append_new_task_and_set_status_to_pending(
        &mut self,
        dag_run_id: &usize,
        function_name: String,
        template_args: Value,
        options: TaskOptions,
        lazy_expand: bool,
        is_dynamic: bool,
        is_branch: bool,
    ) -> usize;
    fn get_template_args(&self, dag_run_id: &usize, task_id: &usize) -> Value;
    fn set_template_args(&mut self, dag_run_id: &usize, task_id: &usize, template_args_str: &str);
}

pub trait DefRunner {
    fn is_task_completed(&mut self, dag_run_id: &usize, task_id: &usize) -> bool;
    fn task_needs_running(&mut self, dag_run_id: &usize, task_id: &usize) -> bool;
    fn get_all_tasks_needs_running(&mut self, dag_run_id: &usize) -> Vec<Task>;

    fn enqueue_run(&mut self, dag_name: &str, dag_hash: &str, logical_date: DateTime<Utc>)
        -> usize;
    fn is_completed(&mut self, dag_run_id: &usize) -> bool;
    fn run(&mut self, dag_run_id: &usize, max_threads: usize);

    fn get_circular_dependencies(
        &self,
        dag_run_id: &usize,
        start_node: usize,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>>;
    fn update_referenced_dependencies(&mut self, dag_run_id: &usize, downstream_task_id: &usize);
    fn attempt_run_task(
        &mut self,
        dag_run_id: &usize,
        task: &Task,
        tx: &Sender<(usize, TaskResult)>,
        max_threads: usize,
        // thread_count: &Arc<Mutex<usize>>,
    ) -> (bool, bool);
    fn resolve_args(
        &mut self,
        dag_run_id: &usize,
        template_args: &Value,
        upstream_deps: &HashMap<(usize, Option<String>), Option<String>>,
    ) -> Result<Value, Error>;
    fn run_dag_local(&mut self, max_threads: usize);
    // fn get_mermaid_graph(&self, dag_run_id: &usize) -> String;
    fn get_graphite_graph(&mut self, dag_run_id: &usize) -> Vec<Value>;
    fn get_tree(
        &self,
        dag_run_id: &usize,
        task_id: &usize,
        depth: usize,
        prefix: &str,
        prev_is_last: Vec<bool>,
        ts: &mut Vec<usize>,
    ) -> String;
    fn handle_task_result(&mut self, dag_run_id: &usize, attempt: TaskResult);
}

impl<U: Runner> DefRunner for U {
    fn is_completed(&mut self, dag_run_id: &usize) -> bool {
        self.get_all_tasks_incomplete(dag_run_id)
            .iter()
            .all(|task| self.is_task_completed(dag_run_id, &task.id))
    }

    fn is_task_completed(&mut self, dag_run_id: &usize, task_id: &usize) -> bool {
        // (self.task_results.contains_key(task_id) && !self.task_results[task_id].needs_retry())
        //     || (self.task_statuses.contains_key(task_id)
        //         && self.task_statuses[task_id] == TaskStatus::Skipped)
        match self.get_task_status(dag_run_id, task_id) {
            TaskStatus::Pending | TaskStatus::Running | TaskStatus::Retrying => false,
            TaskStatus::Success | TaskStatus::Failure | TaskStatus::Skipped => true,
        }
    }

    fn task_needs_running(&mut self, dag_run_id: &usize, task_id: &usize) -> bool {
        // (self.task_results.contains_key(task_id) && !self.task_results[task_id].needs_retry())
        //     || (self.task_statuses.contains_key(task_id)
        //         && self.task_statuses[task_id] == TaskStatus::Skipped)
        // dbg!(&self.get_task_status(dag_run_id, task_id).as_str());
        matches!(
            self.get_task_status(dag_run_id, task_id),
            TaskStatus::Pending
        )
    }

    fn get_all_tasks_needs_running(&mut self, dag_run_id: &usize) -> Vec<Task> {
        self.get_all_tasks_incomplete(dag_run_id)
            .iter()
            .filter(|n| self.task_needs_running(dag_run_id, &n.id))
            .cloned()
            .collect()
    }

    fn enqueue_run(
        &mut self,
        dag_name: &str,
        dag_hash: &str,
        logical_date: DateTime<Utc>,
    ) -> usize {
        let dag_run_id = self.create_new_run(dag_name, dag_hash, logical_date);

        for task in self.get_default_tasks() {
            self.append_new_task_and_set_status_to_pending(
                &dag_run_id,
                // task.name,
                task.function_name,
                task.template_args,
                task.options,
                task.lazy_expand,
                task.is_dynamic,
                task.is_branch,
            );
            self.update_referenced_dependencies(&dag_run_id, &task.id);
        }

        for (upstream_task_id, downstream_task_id) in self.get_default_edges() {
            self.insert_edge(&dag_run_id, &(upstream_task_id, downstream_task_id));
        }
        dag_run_id
    }

    fn handle_task_result(&mut self, dag_run_id: &usize, result: TaskResult) {
        let mut result = result;
        let mut b = Value::Null;
        if result.is_branch {
            b = result.result.clone();
            result.result = result.result["val"].clone();
        }

        self.insert_task_results(dag_run_id, &result);

        result.print_task_result();

        if result.needs_retry() {
            println!(
                "\nattempt failed, retrying {}/{}\n",
                result.attempt + 1,
                result.max_attempts
            );
            self.set_task_status(dag_run_id, &result.task_id, TaskStatus::Retrying);
            return;
        }

        if result.is_branch && result.success {
            let to_skip = if b["is_left"].as_bool().unwrap() {
                result.task_id + 2
            } else {
                result.task_id + 1
            };
            let mut arr = vec![to_skip];
            arr.append(
                &mut self
                    .get_downstream(dag_run_id, &to_skip)
                    .iter()
                    .copied()
                    .collect(),
            );

            while let Some(curr) = arr.pop() {
                arr.append(
                    &mut self
                        .get_downstream(dag_run_id, &curr)
                        .iter()
                        .copied()
                        .collect(),
                );
                // dbg!(curr);
                self.set_task_status(dag_run_id, &curr, TaskStatus::Skipped);
            }
        }

        self.set_task_status(
            dag_run_id,
            &result.task_id,
            if result.success {
                TaskStatus::Success
            } else {
                TaskStatus::Failure
            },
        );
    }

    fn attempt_run_task(
        &mut self,
        dag_run_id: &usize,
        task: &Task,
        tx: &Sender<(usize, TaskResult)>,
        max_threads: usize,
        // thread_count: &Arc<Mutex<usize>>,
    ) -> (bool, bool) {
        // if *Arc::clone(thread_count).lock().unwrap() >= max_threads {
        //     return false;
        // }

        if self.is_task_completed(dag_run_id, &task.id) {
            return (false, true);
        }

        if self.any_upstream_incomplete(dag_run_id, &task.id) {
            return (false, false);
        }

        if !self.set_status_to_running_if_possible(dag_run_id, &task.id) {
            return (false, true);
        }

        let resolution_result = self.resolve_args(
            dag_run_id,
            &task.template_args,
            &self.get_dependency_keys(dag_run_id, &task.id),
        );
        let attempt: usize = self.get_attempt_by_task_id(dag_run_id, &task.id);

        if let Err(resolution_result) = resolution_result {
            self.handle_task_result(
                dag_run_id,
                TaskResult::premature_error(
                    task.id,
                    attempt,
                    task.options.max_attempts,
                    task.function_name.clone(),
                    task.template_args.clone(),
                    resolution_result.to_string(),
                    task.is_branch,
                ),
            );

            return (false, true);
        }

        if task.lazy_expand {
            let downstream = self.get_downstream(dag_run_id, &task.id);

            let mut lazy_ids: Vec<usize> = vec![];

            // only expands json arrays, (expand over maps?)
            for res in resolution_result.as_ref().unwrap().as_array().unwrap() {
                let new_id = self.append_new_task_and_set_status_to_pending(
                    dag_run_id,
                    task.function_name.clone(),
                    res.clone(),
                    task.options,
                    false,
                    true,
                    false,
                );

                lazy_ids.push(new_id);
                self.insert_edge(dag_run_id, &(task.id, new_id));
            }

            if !downstream.is_empty() {
                let function_name = function_name_as_string(&collector).to_string();

                let collector_id = self.append_new_task_and_set_status_to_pending(
                    dag_run_id,
                    function_name,
                    json!(lazy_ids
                        .iter()
                        .map(|id| TaskRef::<Value> {
                            _marker: std::marker::PhantomData,
                            key: None,

                            task_ids: HashSet::from([*id])
                        })
                        .collect::<Vec<TaskRef<Value>>>()),
                    task.options,
                    false,
                    true,
                    false,
                );
                self.update_referenced_dependencies(dag_run_id, &collector_id);

                for lazy_id in &lazy_ids {
                    self.insert_edge(dag_run_id, &(*lazy_id, collector_id));
                }

                for d in &downstream {
                    self.insert_edge(dag_run_id, &(collector_id, *d));

                    self.set_template_args(
                        dag_run_id,
                        d,
                        &serde_json::to_string(&self.get_template_args(dag_run_id, d))
                            .unwrap()
                            .replace(
                                &serde_json::to_string(&TaskRef::<Value> {
                                    _marker: std::marker::PhantomData,
                                    key: None,

                                    task_ids: HashSet::from([task.id]),
                                })
                                .unwrap(),
                                &serde_json::to_string(&TaskRef::<Value> {
                                    _marker: std::marker::PhantomData,
                                    key: None,

                                    task_ids: HashSet::from([collector_id]),
                                })
                                .unwrap(),
                            ),
                    );
                    self.update_referenced_dependencies(dag_run_id, d);
                }
                for d in &downstream {
                    self.remove_edge(dag_run_id, &(task.id, *d));
                }
            }

            let start = Utc::now();

            tx.send((
                *dag_run_id,
                TaskResult {
                    task_id: task.id,
                    result: Value::Null,
                    attempt,
                    max_attempts: task.options.max_attempts,
                    function_name: task.function_name.clone(),
                    success: true,
                    stdout: "".into(),
                    stderr: "".into(),
                    template_args_str: "".into(),
                    resolved_args_str: "".into(),
                    started: start.to_rfc3339(),
                    ended: start.to_rfc3339(),
                    elapsed: 0,
                    premature_failure: false,
                    premature_failure_error_str: "".into(),
                    is_branch: task.is_branch,
                },
            ))
            .unwrap();
            return (true, true);
        }

        // *thread_count.lock().unwrap() += 1;
        let task_handle = task.execute(
            *dag_run_id,
            self.get_dag_name(),
            resolution_result.unwrap(),
            attempt,
            tx,
        );

        if max_threads == 1 {
            task_handle.join().unwrap();
        }
        return (true, true);
    }

    fn resolve_args(
        &mut self,
        dag_run_id: &usize,
        template_args: &Value,
        upstream_deps: &HashMap<(usize, Option<String>), Option<String>>,
    ) -> Result<Value, Error> {
        let mut results: HashMap<usize, Value> = HashMap::new();
        for ((upstream_task_id, _), key) in upstream_deps {
            if results.contains_key(upstream_task_id) {
                continue;
            }

            if !self.is_task_completed(dag_run_id, upstream_task_id) {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("upstream task_id {} does not exist!", upstream_task_id),
                ));
            }
            let task_result = self.get_task_result(dag_run_id, upstream_task_id);
            results.insert(*upstream_task_id, task_result.result.clone());

            if !task_result.success {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("upstream task_id {} failed!", upstream_task_id,),
                ));
            }

            if key.is_none() {
                continue;
            }

            if task_result.result.is_object() {
                let upstream_results_map = task_result.result.as_object().unwrap();
                let key = key.as_ref().unwrap();

                if !upstream_results_map.contains_key(key) {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!(
                            "upstream task_id {} result does not contain key {}",
                            upstream_task_id, key
                        ),
                    ));
                }

                continue;
            }

            return Err(Error::new(
                ErrorKind::NotFound,
                format!(
                    "upstream_task_id {} result type '{:?}' is not a map",
                    upstream_task_id, task_result.result
                ),
            ));
        }

        let mut resolved_args: Value = template_args.clone();
        if template_args.is_array() {
            return Ok(json!(template_args
                .as_array()
                .unwrap()
                .iter()
                .map(|a| {
                    if !a.is_object() {
                        return a.clone();
                    }
                    let binding = a.clone();
                    let map: &serde_json::Map<String, Value> = binding.as_object().unwrap();

                    if !map.contains_key("upstream_task_id") {
                        return a.clone();
                    }

                    let upstream_task_id = map["upstream_task_id"].as_u64().unwrap() as usize;
                    let result: Value = results[&upstream_task_id].clone();
                    if map.contains_key("key") {
                        return result.as_object().unwrap()[map["key"].as_str().unwrap()].clone();
                    } else {
                        return result;
                    }
                })
                .collect::<Vec<Value>>()));
        } else if template_args.is_object() {
            let map = template_args.as_object().unwrap();

            for (k, v) in map {
                resolved_args[k.to_string()] = v.clone();
            }

            for ((upstream_task_id, original_key), key) in upstream_deps {
                let result = &results[upstream_task_id];

                if key.is_none() {
                    if original_key.is_some() {
                        resolved_args[original_key.clone().unwrap().to_string()] = result.clone();
                    } else {
                        resolved_args = result.clone();
                        break;
                    }
                    continue;
                }

                let upstream_results_map = result.as_object().unwrap();
                let key = key.as_ref().unwrap();

                if original_key.is_some() {
                    resolved_args[original_key.clone().unwrap().to_string()] =
                        upstream_results_map[key].clone();
                } else {
                    resolved_args = upstream_results_map[key].clone();
                }
            }
        }
        Ok(resolved_args)
    }

    fn run(&mut self, dag_run_id: &usize, max_threads: usize) {
        let mut thread_count = 0usize;
        let tasks = self.get_all_tasks_needs_running(dag_run_id);
        let mut tasks_map: HashMap<usize, &Task> = HashMap::new();
        let mut task_ids: HashSet<usize> = HashSet::new();
        let mut downstream_ids: HashMap<usize, HashSet<usize>> = HashMap::new();

        for task in &tasks {
            tasks_map.insert(task.id, &task);
            task_ids.insert(task.id);

            if !task.lazy_expand && !task.is_dynamic {
                downstream_ids.insert(task.id, self.get_downstream(dag_run_id, &task.id));
            }
        }

        let tasks_map = tasks_map;
        let downstream_ids = downstream_ids;

        let (tx, rx) = mpsc::channel::<(usize, TaskResult)>();

        for task_id in &task_ids.clone() {
            let task = tasks_map[task_id];
            let (spawned_thread, run_attempted) =
                self.attempt_run_task(dag_run_id, task, &tx.clone(), max_threads);

            if spawned_thread {
                thread_count += 1;
            }

            if run_attempted {
                task_ids.remove(task_id);
            }

            if thread_count >= max_threads {
                break;
            }
        }

        if thread_count == 0 {
            drop(tx);
            return;
        }

        'outer: for (run_id, received) in &rx {
            if thread_count >= 1 {
                thread_count -= 1;
            }
            let task_id: usize = received.task_id;
            self.handle_task_result(&run_id, received);

            // retry run if task failed
            if self.task_needs_running(&run_id, &task_id) {
                let (spawned_thread, run_attempted) =
                    self.attempt_run_task(&run_id, &tasks[task_id], &tx.clone(), max_threads);
                if spawned_thread {
                    thread_count += 1;
                }
                if run_attempted {
                    task_ids.remove(&task_id);
                }
                if thread_count >= max_threads {
                    continue 'outer;
                }
            } else {
                if downstream_ids.contains_key(&task_id) {
                    for downstream_task_id in downstream_ids[&task_id].iter() {
                        let (spawned_thread, run_attempted) = self.attempt_run_task(
                            &run_id,
                            &tasks[*downstream_task_id],
                            &tx.clone(),
                            max_threads,
                        );
                        if spawned_thread {
                            thread_count += 1;
                        }
                        if run_attempted {
                            task_ids.remove(&task_id);
                        }
                        if thread_count >= max_threads {
                            continue 'outer;
                        }
                    }
                } else {
                    for downstream_task_id in self.get_downstream(&run_id, &task_id).iter() {
                        let (spawned_thread, run_attempted) = self.attempt_run_task(
                            &run_id,
                            &self.get_task_by_id(&run_id, downstream_task_id),
                            &tx.clone(),
                            max_threads,
                        );
                        if spawned_thread {
                            thread_count += 1;
                        }
                        if run_attempted {
                            task_ids.remove(&task_id);
                        }
                        if thread_count >= max_threads {
                            continue 'outer;
                        }
                    }
                }
            }

            for task_id in &task_ids.clone() {
                let task = tasks_map[task_id];

                let (spawned_thread, run_attempted) =
                    self.attempt_run_task(dag_run_id, task, &tx.clone(), max_threads);

                if spawned_thread {
                    thread_count += 1;
                }
                if run_attempted {
                    task_ids.remove(task_id);
                }

                if thread_count >= max_threads {
                    continue 'outer;
                }
            }

            // no more running threads so drop sender
            if thread_count == 0 {
                drop(tx);
                self.mark_finished(dag_run_id);
                return;
            }
        }
    }

    fn run_dag_local(&mut self, max_threads: usize) {
        let hash = hash_dag(
            &serde_json::to_string(&self.get_default_tasks()).unwrap(),
            &self
                .get_default_edges()
                .iter()
                .collect::<Vec<&(usize, usize)>>(),
        );
        let dag_run_id = &self.enqueue_run(&self.get_dag_name(), &hash, Utc::now());

        for task in self.get_default_tasks() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            let deps = self.get_circular_dependencies(dag_run_id, task.id, &mut visited, &mut path);

            if let Some(deps) = deps {
                panic!("{:?}", deps);
            }
        }

        self.run(dag_run_id, max_threads);
    }

    fn get_circular_dependencies(
        &self,
        dag_run_id: &usize,
        start_node: usize,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        visited.insert(start_node);
        path.push(start_node);

        for neighbor in self.get_upstream(dag_run_id, &start_node) {
            if !visited.contains(&neighbor) {
                if let Some(cycle) =
                    self.get_circular_dependencies(dag_run_id, neighbor, visited, path)
                {
                    return Some(cycle);
                }
            } else if path.contains(&neighbor) {
                // Circular dependency detected
                let mut cycle = path.clone();
                cycle.push(neighbor);
                return Some(cycle);
            }
        }

        path.pop();
        visited.remove(&start_node);
        None
    }

    // fn get_mermaid_graph(&self, dag_run_id: &usize) -> String {
    //     let task_statuses: Vec<(String, TaskStatus)> = self
    //         .get_all_tasks(dag_run_id)
    //         .iter()
    //         .map(|t| {
    //             (
    //                 t.function_name.clone(),
    //                 self.get_task_status(dag_run_id, &t.id),
    //             )
    //         })
    //         .collect();

    //     let mut out = "".to_string();
    //     out += "flowchart TD\n";

    //     for (task_id, (function_name, task_status)) in task_statuses.iter().enumerate() {
    //         let styling = get_styling_for_status(task_status);
    //         out += &format!("  id{task_id}({function_name}_{task_id})\n");
    //         out += &format!("  style id{task_id} {styling}\n");

    //         for edge_id in self.get_upstream(dag_run_id, &task_id) {
    //             out += &format!("  id{edge_id}-->id{task_id}\n");
    //         }
    //     }

    //     out
    // }

    fn get_graphite_graph(&mut self, dag_run_id: &usize) -> Vec<Value> {
        let task_statuses: Vec<(String, TaskStatus)> = self
            .get_all_tasks(dag_run_id)
            .iter()
            .map(|t| {
                (
                    t.function_name.clone(),
                    self.get_task_status(dag_run_id, &t.id),
                )
            })
            .collect();

        // let mut out = "".to_string();
        // out += "flowchart TD\n";

        // const presetComplex = '['
        // '{"id":"A","next":[{"outcome":"B","type":"one"}]},'
        // '{"id":"U","next":[{"outcome":"G","type":"one"}]},'
        // '{"id":"B","next":[{"outcome":"C","type":"one"},{"outcome":"D","type":"one"},{"outcome":"E","type":"one"},{"outcome":"F","type":"one"},{"outcome":"M","type":"one"}]},'
        // '{"id":"C","next":[{"outcome":"G","type":"one"}]},'
        // '{"id":"D","next":[{"outcome":"H","type":"one"}]},'
        // '{"id":"E","next":[{"outcome":"H","type":"one"}]},'
        // '{"id":"F","next":[{"outcome":"W","type":"one"},{"outcome":"N","type":"one"},{"outcome":"O","type":"one"}]},'
        // '{"id":"W","next":[]},'
        // '{"id":"N","next":[{"outcome":"I","type":"one"}]},'
        // '{"id":"O","next":[{"outcome":"P","type":"one"}]},'
        // '{"id":"P","next":[{"outcome":"I","type":"one"}]},'
        // '{"id":"M","next":[{"outcome":"L","type":"one"}]},'
        // '{"id":"G","next":[{"outcome":"I","type":"one"}]},'
        // '{"id":"H","next":[{"outcome":"J","type":"one"}]},'
        // '{"id":"I","next":[]},'
        // '{"id":"J","next":[{"outcome":"K","type":"one"}]},'
        // '{"id":"K","next":[{"outcome":"L","type":"one"}]},'
        // '{"id":"L","next":[]}'
        // ']';

        task_statuses
            .iter()
            .enumerate()
            .map(|(task_id, (function_name, task_status))| {
                // let styling = get_styling_for_status(task_status);
                // out += &format!("  id{task_id}({function_name}_{task_id})\n");
                // out += &format!("  style id{task_id} {styling}\n");

                // let node_input =

                // for edge_id in self.get_upstream(dag_run_id, &task_id) {
                //     // out += &format!("  id{edge_id}-->id{task_id}\n");
                // }
                let name = format!("{function_name}_{task_id}");
                let mut downstream = Vec::from_iter(self.get_downstream(dag_run_id, &task_id));
                downstream.sort();

                let next = downstream
                    .iter()
                    .map(|f| json!({"outcome": f.to_string()}))
                    .collect::<Vec<Value>>();
                json!({
                    "id": task_id.to_string(),
                    "name": name,
                    "next": next,
                    "status": task_status.as_str(),
                })
            })
            .collect()
    }

    fn update_referenced_dependencies(&mut self, dag_run_id: &usize, downstream_task_id: &usize) {
        let template_args = self.get_template_args(dag_run_id, downstream_task_id);

        if template_args.is_array() {
            for value in template_args
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v.is_object())
            {
                let map_value = value.as_object().unwrap();
                if map_value.contains_key("upstream_task_id") {
                    let upstream_task_id =
                        map_value.get("upstream_task_id").unwrap().as_u64().unwrap() as usize;

                    self.set_dependency_keys(
                        dag_run_id,
                        downstream_task_id,
                        (upstream_task_id, None),
                        if map_value.contains_key("key") {
                            Some(map_value.get("key").unwrap().as_str().unwrap().to_string())
                        } else {
                            None
                        },
                    );
                    self.insert_edge(dag_run_id, &(upstream_task_id, *downstream_task_id));
                }
            }
        } else if template_args.is_object() {
            let template_args = template_args.as_object().unwrap();
            if template_args.contains_key("upstream_task_id") {
                let upstream_task_id = template_args
                    .get("upstream_task_id")
                    .unwrap()
                    .as_u64()
                    .unwrap() as usize;
                self.set_dependency_keys(
                    dag_run_id,
                    downstream_task_id,
                    (upstream_task_id, None),
                    if template_args.contains_key("key") {
                        Some(
                            template_args
                                .get("key")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string(),
                        )
                    } else {
                        None
                    },
                );

                self.insert_edge(dag_run_id, &(upstream_task_id, *downstream_task_id));

                return;
            }

            for (key, value) in template_args.iter().filter(|(_, v)| v.is_object()) {
                let map = value.as_object().unwrap();
                if map.contains_key("upstream_task_id") {
                    let upstream_task_id =
                        map.get("upstream_task_id").unwrap().as_u64().unwrap() as usize;

                    self.set_dependency_keys(
                        dag_run_id,
                        downstream_task_id,
                        (upstream_task_id, Some(key.to_string())),
                        if map.contains_key("key") {
                            Some(map.get("key").unwrap().as_str().unwrap().to_string())
                        } else {
                            None
                        },
                    );

                    self.insert_edge(dag_run_id, &(upstream_task_id, *downstream_task_id));
                }
            }
        }
    }

    fn get_tree(
        &self,
        dag_run_id: &usize,
        task_id: &usize,
        _depth: usize,
        prefix: &str,
        prev_is_last: Vec<bool>,
        ts: &mut Vec<usize>,
    ) -> String {
        let binding = self.get_downstream(dag_run_id, task_id);
        let mut children: Vec<&usize> = binding.iter().collect();
        children.sort();
        let mut output = format!(
            "{}{}_{}\n",
            prefix,
            self.get_task_by_id(dag_run_id, task_id).function_name,
            task_id,
        );

        for (index, child) in children.iter().enumerate() {
            let is_last = index == children.len() - 1;
            let child_prefix = prev_is_last.iter().fold(String::new(), |acc, &last| {
                if last {
                    acc + "    "
                } else {
                    acc + "│   "
                }
            });

            let connector = if is_last { "└── " } else { "├── " };
            let mut new_prev_is_last = prev_is_last.clone();
            new_prev_is_last.push(is_last);
            ts.push(**child);
            output.push_str(&self.get_tree(
                dag_run_id,
                child,
                _depth + 1,
                &format!("{}{}", child_prefix, connector),
                new_prev_is_last,
                ts,
            ));
        }

        output
    }
}

// fn get_styling_for_status(task_status: &TaskStatus) -> String {
//     match task_status {
//         TaskStatus::Pending => "color:black,stroke:grey,fill:white,stroke-width:4px".into(),
//         TaskStatus::Success => "color:black,stroke:green,fill:white,stroke-width:4px".into(),
//         TaskStatus::Failure => "color:black,stroke:red,fill:white,stroke-width:4px".into(),
//         TaskStatus::Running => "color:black,stroke:#90EE90,fill:white,stroke-width:4px".into(),
//         TaskStatus::Retrying => "color:black,stroke:orange,fill:white,stroke-width:4px".into(),
//         TaskStatus::Skipped => "color:black,stroke:pink,fill:white,stroke-width:4px".into(),
//     }
// }

pub fn collector(args: Value) -> Value {
    args
}
