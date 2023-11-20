use std::{
    collections::{HashMap, HashSet},
    io::{Error, ErrorKind},
};

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use task::{
    ordered_queued_task::OrderedQueuedTask, task_ref_inner::TaskRefInner, task_result::TaskResult,
    task_status::TaskStatus, Task,
};
use utils::{collector, function_name_as_string};

use crate::Runner;

pub trait BlanketRunner {
    fn is_task_completed(&mut self, run_id: usize, task_id: usize) -> bool;
    fn task_needs_running(&mut self, run_id: usize, task_id: usize) -> bool;
    fn enqueue_run(&mut self, dag_name: &str, dag_hash: &str, logical_date: DateTime<Utc>)
        -> usize;
    fn work(&mut self, run_id: usize, queued_task: OrderedQueuedTask, executable_path: &str);
    fn get_circular_dependencies(
        &self,
        run_id: usize,
        start_node: usize,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>>;
    fn update_referenced_dependencies(&mut self, run_id: usize, downstream_task_id: usize);
    fn run_task(
        &mut self,
        run_id: usize,
        task: &Task,
        attempt: usize,
        resolution_result: &Value,
        executable_path: &str,
    ) -> TaskResult;
    fn resolve_args(
        &mut self,
        run_id: usize,
        template_args: &Value,
        upstream_deps: &HashMap<(usize, String), String>,
    ) -> Result<Value, Error>;
    fn get_graphite_graph(&mut self, run_id: usize) -> Vec<Value>;
    fn get_tree(
        &self,
        run_id: usize,
        task_id: usize,
        depth: usize,
        prefix: &str,
        prev_is_last: Vec<bool>,
        task_ids_in_order: &mut Vec<usize>,
    ) -> String;
    fn handle_task_result(&mut self, run_id: usize, result: TaskResult);
}

impl<U: Runner + Send + Sync> BlanketRunner for U {
    fn is_task_completed(&mut self, run_id: usize, task_id: usize) -> bool {
        match self.get_task_status(run_id, task_id) {
            TaskStatus::Pending | TaskStatus::Running | TaskStatus::Retrying => false,
            TaskStatus::Success | TaskStatus::Failure | TaskStatus::Skipped => true,
        }
    }

    fn task_needs_running(&mut self, run_id: usize, task_id: usize) -> bool {
        matches!(
            self.get_task_status(run_id, task_id),
            TaskStatus::Pending | TaskStatus::Retrying
        )
    }

    fn enqueue_run(
        &mut self,
        dag_name: &str,
        dag_hash: &str,
        logical_date: DateTime<Utc>,
    ) -> usize {
        let run_id = self.create_new_run(dag_name, dag_hash, logical_date);
        let default_tasks = self.get_default_tasks();
        for task in &default_tasks {
            self.append_new_task_and_set_status_to_pending(
                run_id,
                &task.function_name,
                &task.template_args,
                &task.options,
                task.lazy_expand,
                task.is_dynamic,
                task.is_branch,
            );
            self.update_referenced_dependencies(run_id, task.id);
        }

        for (upstream_task_id, downstream_task_id) in self.get_default_edges() {
            self.insert_edge(run_id, (upstream_task_id, downstream_task_id));
        }

        for task in default_tasks
            .iter()
            .filter(|task| self.get_task_depth(run_id, task.id) == 0)
            .collect::<Vec<&Task>>()
        {
            self.enqueue_task(run_id, task.id);
        }

        run_id
    }

    fn handle_task_result(&mut self, run_id: usize, result: TaskResult) {
        let mut result = result;
        let mut branch_left = false;

        if result.is_branch {
            branch_left = result.result["is_left"].as_bool().unwrap();
            result.result = result.result["val"].take();
        }

        self.insert_task_results(run_id, &result);

        result.print_task_result(
            self.get_template_args(run_id, result.task_id),
            self.get_log(run_id, result.task_id, result.attempt),
        );

        if result.needs_retry() {
            println!(
                "\nattempt failed, retrying {}/{}\n",
                result.attempt + 1,
                result.max_attempts
            );
            self.set_task_status(run_id, result.task_id, TaskStatus::Retrying);
            return;
        }

        if result.is_branch && result.success {
            let skip_task = if branch_left {
                result.task_id + 2
            } else {
                result.task_id + 1
            };
            let mut to_skip = vec![skip_task];
            to_skip.append(&mut self.get_downstream(run_id, skip_task));

            while let Some(curr) = to_skip.pop() {
                to_skip.append(&mut self.get_downstream(run_id, curr));
                self.set_task_status(run_id, curr, TaskStatus::Skipped);
            }
        }

        self.set_task_status(
            run_id,
            result.task_id,
            if result.success {
                TaskStatus::Success
            } else {
                TaskStatus::Failure
            },
        );
    }

    fn run_task(
        &mut self,
        run_id: usize,
        task: &Task,
        attempt: usize,
        resolution_result: &Value,
        executable_path: &str,
    ) -> TaskResult {
        if task.lazy_expand {
            let downstream = self.get_downstream(run_id, task.id);

            let mut lazy_ids = vec![];
            for res in resolution_result.as_array().unwrap() {
                let new_id = self.append_new_task_and_set_status_to_pending(
                    run_id,
                    &task.function_name,
                    res,
                    &task.options,
                    false,
                    true,
                    false,
                );

                lazy_ids.push(new_id);
                self.insert_edge(run_id, (task.id, new_id));
            }

            if !downstream.is_empty() {
                let function_name = function_name_as_string(&collector).to_string();

                let collector_id = self.append_new_task_and_set_status_to_pending(
                    run_id,
                    &function_name,
                    &json!(lazy_ids
                        .iter()
                        .map(|id| TaskRefInner::<Value> {
                            _marker: std::marker::PhantomData,
                            key: None,
                            task_ids: HashSet::from([*id])
                        })
                        .collect::<Vec<TaskRefInner<Value>>>()),
                    &task.options,
                    false,
                    true,
                    false,
                );
                self.update_referenced_dependencies(run_id, collector_id);

                for lazy_id in &lazy_ids {
                    self.insert_edge(run_id, (*lazy_id, collector_id));
                }

                for d in &downstream {
                    self.insert_edge(run_id, (collector_id, *d));

                    self.set_template_args(
                        run_id,
                        *d,
                        &serde_json::to_string(&self.get_template_args(run_id, *d))
                            .unwrap()
                            .replace(
                                &serde_json::to_string(&TaskRefInner::<Value> {
                                    _marker: std::marker::PhantomData,
                                    key: None,
                                    task_ids: HashSet::from([task.id]),
                                })
                                .unwrap(),
                                &serde_json::to_string(&TaskRefInner::<Value> {
                                    _marker: std::marker::PhantomData,
                                    key: None,
                                    task_ids: HashSet::from([collector_id]),
                                })
                                .unwrap(),
                            ),
                    );
                }
                for d in &downstream {
                    self.remove_edge(run_id, (task.id, *d));
                    self.update_referenced_dependencies(run_id, *d);
                    self.delete_task_depth(run_id, *d);
                    self.enqueue_task(run_id, *d);
                }

                self.enqueue_task(run_id, collector_id);
            }
            for lazy_id in &lazy_ids {
                self.enqueue_task(run_id, *lazy_id);
            }

            let start = Utc::now();

            return TaskResult {
                task_id: task.id,
                result: Value::Null,
                attempt,
                max_attempts: task.options.max_attempts,
                function_name: task.function_name.clone(),
                success: true,
                resolved_args_str: "".into(),
                started: start.to_rfc3339(),
                ended: start.to_rfc3339(),
                elapsed: 0,
                premature_failure: false,
                premature_failure_error_str: "".into(),
                is_branch: task.is_branch,
            };
        }

        task.execute(
            resolution_result,
            attempt,
            self.get_log_handle_closure(run_id, task.id, attempt),
            self.get_log_handle_closure(run_id, task.id, attempt),
            executable_path,
        )
    }

    fn resolve_args(
        &mut self,
        run_id: usize,
        template_args: &Value,
        upstream_deps: &HashMap<(usize, String), String>,
    ) -> Result<Value, Error> {
        let mut results: HashMap<usize, Value> = HashMap::new();
        for ((upstream_task_id, _), key) in upstream_deps {
            if results.contains_key(upstream_task_id) {
                continue;
            }

            if !self.is_task_completed(run_id, *upstream_task_id) {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("upstream task_id {} does not exist!", upstream_task_id),
                ));
            }
            let task_result = self.get_task_result(run_id, *upstream_task_id);
            results.insert(*upstream_task_id, task_result.result.clone());
            if !task_result.success {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("upstream task_id {} failed!", upstream_task_id,),
                ));
            }

            if key.is_empty() {
                continue;
            }

            if task_result.result.is_object() {
                let upstream_results_map = task_result.result.as_object().unwrap();

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
                        result.as_object().unwrap()[map["key"].as_str().unwrap()].clone()
                    } else {
                        result
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

                if key.is_empty() {
                    if !original_key.is_empty() {
                        resolved_args[original_key] = result.clone();
                    } else {
                        resolved_args = result.clone();
                        break;
                    }
                    continue;
                }

                let upstream_results_map = result.as_object().unwrap();

                if !original_key.is_empty() {
                    resolved_args[original_key] = upstream_results_map[key].clone();
                } else {
                    resolved_args = upstream_results_map[key].clone();
                }
            }
        }
        Ok(resolved_args)
    }

    fn work(
        &mut self,
        run_id: usize,
        ordered_queued_task: OrderedQueuedTask,
        executable_path: &str,
    ) {
        if self.is_task_completed(run_id, ordered_queued_task.queued_task.task_id) {
            return;
        }
        if self.any_upstream_incomplete(run_id, ordered_queued_task.queued_task.task_id) {
            self.enqueue_task(run_id, ordered_queued_task.queued_task.task_id);
            return;
        }

        let task = self.get_task_by_id(run_id, ordered_queued_task.queued_task.task_id);
        let deps = self.get_dependency_keys(run_id, task.id);
        let resolution_result = self.resolve_args(run_id, &task.template_args, &deps);
        let attempt: usize = self.get_attempt_by_task_id(run_id, task.id);

        match resolution_result {
            Ok(resolution_result) => {
                let result =
                    self.run_task(run_id, &task, attempt, &resolution_result, executable_path);

                self.handle_task_result(run_id, result);

                if self.task_needs_running(run_id, ordered_queued_task.queued_task.task_id) {
                    self.enqueue_task(run_id, ordered_queued_task.queued_task.task_id);
                } else {
                    for downstream in self
                        .get_downstream(run_id, ordered_queued_task.queued_task.task_id)
                        .iter()
                        .filter(|d| {
                            !self.is_task_completed(run_id, **d)
                                && !self.any_upstream_incomplete(run_id, **d)
                        })
                        .collect::<Vec<&usize>>()
                    {
                        self.enqueue_task(run_id, *downstream);
                    }
                }
            }
            Err(resolution_result) => {
                self.handle_task_result(
                    run_id,
                    TaskResult::premature_error(
                        task.id,
                        attempt,
                        task.options.max_attempts,
                        task.function_name.clone(),
                        resolution_result.to_string(),
                        task.is_branch,
                    ),
                );
                for downstream in self
                    .get_downstream(run_id, ordered_queued_task.queued_task.task_id)
                    .iter()
                    .filter(|d| {
                        !self.is_task_completed(run_id, **d)
                            && !self.any_upstream_incomplete(run_id, **d)
                    })
                    .collect::<Vec<&usize>>()
                {
                    self.enqueue_task(run_id, *downstream);
                }
            }
        }
    }

    fn get_circular_dependencies(
        &self,
        run_id: usize,
        start_node: usize,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        visited.insert(start_node);
        path.push(start_node);

        for neighbor in self.get_upstream(run_id, start_node) {
            if !visited.contains(&neighbor) {
                if let Some(cycle) = self.get_circular_dependencies(run_id, neighbor, visited, path)
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

    fn get_graphite_graph(&mut self, run_id: usize) -> Vec<Value> {
        let task_statuses: Vec<(usize, String, TaskStatus)> = self
            .get_all_tasks(run_id)
            .iter()
            .map(|task| {
                (
                    task.id,
                    task.function_name.clone(),
                    self.get_task_status(run_id, task.id),
                )
            })
            .collect();

        task_statuses
            .iter()
            .map(|(task_id, function_name, task_status)| {
                let name = format!("{function_name}_{task_id}");
                let next = self
                    .get_downstream(run_id, *task_id)
                    .iter()
                    .map(|downstream_id| json!({"outcome": downstream_id.to_string()}))
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

    fn update_referenced_dependencies(&mut self, run_id: usize, downstream_task_id: usize) {
        let template_args = self.get_template_args(run_id, downstream_task_id);

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
                        run_id,
                        downstream_task_id,
                        (upstream_task_id, "".into()),
                        if map_value.contains_key("key") {
                            map_value.get("key").unwrap().as_str().unwrap().to_string()
                        } else {
                            "".into()
                        },
                    );
                    self.insert_edge(run_id, (upstream_task_id, downstream_task_id));
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
                    run_id,
                    downstream_task_id,
                    (upstream_task_id, "".into()),
                    if template_args.contains_key("key") {
                        template_args
                            .get("key")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                    } else {
                        "".into()
                    },
                );

                self.insert_edge(run_id, (upstream_task_id, downstream_task_id));

                return;
            }

            for (key, value) in template_args.iter().filter(|(_, v)| v.is_object()) {
                let map = value.as_object().unwrap();
                if map.contains_key("upstream_task_id") {
                    let upstream_task_id =
                        map.get("upstream_task_id").unwrap().as_u64().unwrap() as usize;

                    self.set_dependency_keys(
                        run_id,
                        downstream_task_id,
                        (upstream_task_id, key.to_string()),
                        if map.contains_key("key") {
                            map.get("key").unwrap().as_str().unwrap().to_string()
                        } else {
                            "".into()
                        },
                    );

                    self.insert_edge(run_id, (upstream_task_id, downstream_task_id));
                }
            }
        }
    }

    fn get_tree(
        &self,
        run_id: usize,
        task_id: usize,
        _depth: usize,
        prefix: &str,
        prev_is_last: Vec<bool>,
        task_ids_in_order: &mut Vec<usize>,
    ) -> String {
        let children: Vec<usize> = self.get_downstream(run_id, task_id);
        let mut output = format!(
            "{}{}_{}\n",
            prefix,
            self.get_task_by_id(run_id, task_id).function_name,
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
            task_ids_in_order.push(*child);
            output.push_str(&self.get_tree(
                run_id,
                *child,
                _depth + 1,
                &format!("{}{}", child_prefix, connector),
                new_prev_is_last,
                task_ids_in_order,
            ));
        }

        output
    }
}
