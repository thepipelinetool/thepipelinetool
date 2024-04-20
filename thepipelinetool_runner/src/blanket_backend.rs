use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
};

use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use thepipelinetool_task::{
    queued_task::QueuedTask, task_ref_inner::TaskRefInner, task_result::TaskResult,
    task_status::TaskStatus, temp_queued_task::TempQueuedTask, trigger_rule::TriggerRule, Task,
};
use thepipelinetool_utils::{
    collector, function_name_as_string, UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY,
};

use crate::{
    run::{Run, RunStatus},
    Backend,
};
use anyhow::Result;

pub trait BlanketBackend {
    fn trigger_rules_satisfied(&mut self, run_id: usize, task_id: usize) -> Result<bool>;

    fn get_run_status(&mut self, run_id: usize) -> Result<RunStatus>;

    fn is_task_done(&mut self, run_id: usize, task_id: usize) -> Result<bool>;
    fn task_needs_running(&mut self, run_id: usize, task_id: usize) -> Result<bool>;
    fn enqueue_run(&mut self, run: &Run, trigger_params: Option<Value>) -> Result<()>;
    // TODO move tpt_path into OrderedQueuedTask?
    fn work<D: AsRef<OsStr>>(&mut self, queued_task: &TempQueuedTask, tpt_path: D) -> Result<()>;
    fn update_referenced_dependencies(&mut self, run_id: usize, downstream_id: usize)
        -> Result<()>;
    fn run_task<D: AsRef<OsStr>>(
        &mut self,
        run_id: usize,
        task: &Task,
        attempt: usize,
        resolution_result: &Value,
        tpt_path: D,
        scheduled_date_for_run: DateTime<Utc>,
    ) -> Result<TaskResult>;
    fn resolve_args(
        &mut self,
        run_id: usize,
        template_args: &Value,
        upstream_deps: &HashMap<(usize, String), String>,
    ) -> Result<Value>;

    fn handle_task_result(
        &mut self,
        run_id: usize,
        queued_task: &QueuedTask,
        result: TaskResult,
    ) -> Result<()>;
}

impl<U: Backend + Send + Sync> BlanketBackend for U {
    fn get_run_status(&mut self, run_id: usize) -> Result<RunStatus> {
        let mut pending_count = 0;
        let tasks = self.get_all_tasks(run_id)?;

        for task in &tasks {
            let status = self.get_task_status(run_id, task.id)?;

            match status {
                TaskStatus::Failure => return Ok(RunStatus::Failed),
                TaskStatus::Pending | TaskStatus::RetryPending => {
                    pending_count += 1;
                }
                _ => {}
            };
        }
        if pending_count == tasks.len() {
            Ok(RunStatus::Pending)
        } else if pending_count > 0 {
            Ok(RunStatus::Running)
        } else {
            Ok(RunStatus::Success)
        }
    }
    fn trigger_rules_satisfied(&mut self, run_id: usize, task_id: usize) -> Result<bool> {
        let task = self.get_task_by_id(run_id, task_id)?;

        let required_upstream_ids: HashSet<usize> = HashSet::from_iter(
            self.get_dependencies(run_id, task_id)?
                .iter()
                .map(|((u, _), _)| *u),
        );

        // ensure required tasks are done
        for required_upstream_id in &required_upstream_ids {
            if !self.is_task_done(run_id, *required_upstream_id)? {
                return Ok(false);
            }
        }

        match task.options.trigger_rule {
            TriggerRule::AllSuccess => {
                for upstream_id in self.get_upstream(run_id, task_id)? {
                    if matches!(
                        self.get_task_status(run_id, upstream_id)?,
                        TaskStatus::Success
                    ) {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TriggerRule::AllFailed => {
                for upstream_id in self.get_upstream(run_id, task_id)? {
                    if !matches!(
                        self.get_task_status(run_id, upstream_id)?,
                        TaskStatus::Failure
                    ) {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TriggerRule::AllDone => {
                for upstream_id in self.get_upstream(run_id, task_id)? {
                    if required_upstream_ids.contains(&upstream_id) {
                        continue;
                    }
                    if !self.is_task_done(run_id, upstream_id)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TriggerRule::AnyFailed => {
                for upstream_id in self.get_upstream(run_id, task_id)? {
                    if matches!(
                        self.get_task_status(run_id, upstream_id)?,
                        TaskStatus::Failure
                    ) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            TriggerRule::AnyDone => {
                if !required_upstream_ids.is_empty() {
                    return Ok(true);
                }
                for upstream_id in self.get_upstream(run_id, task_id)? {
                    if self.is_task_done(run_id, upstream_id)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            TriggerRule::AnySuccess => {
                for upstream_id in self.get_upstream(run_id, task_id)? {
                    if matches!(
                        self.get_task_status(run_id, upstream_id)?,
                        TaskStatus::Success
                    ) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    fn is_task_done(&mut self, run_id: usize, task_id: usize) -> Result<bool> {
        Ok(match self.get_task_status(run_id, task_id)? {
            TaskStatus::Pending | TaskStatus::Running | TaskStatus::RetryPending => false,
            TaskStatus::Success | TaskStatus::Failure | TaskStatus::Skipped => true,
        })
    }

    fn task_needs_running(&mut self, run_id: usize, task_id: usize) -> Result<bool> {
        Ok(matches!(
            self.get_task_status(run_id, task_id)?,
            TaskStatus::Pending | TaskStatus::RetryPending
        ))
    }

    fn enqueue_run(
        &mut self,
        run: &Run,
        // run_id: usize,
        // pipeline_name: &str,
        // pipeline_hash: &str,
        // scheduled_date_for_run: DateTime<Utc>,
        trigger_params: Option<Value>,
    ) -> Result<()> {
        let default_tasks = self.get_default_tasks()?;
        let trigger_params = trigger_params.unwrap_or(Value::Null);

        for task in &default_tasks {
            let _ = self.append_new_task_and_set_status_to_pending(
                run.run_id,
                &task.name,
                &task.function,
                if task.use_trigger_params {
                    &trigger_params
                } else {
                    &task.template_args
                },
                &task.options,
                task.lazy_expand,
                task.is_dynamic,
                task.is_branch,
                task.use_trigger_params,
            )?;
            self.update_referenced_dependencies(run.run_id, task.id)?;
        }

        for (upstream_id, downstream_id) in self.get_default_edges()? {
            self.insert_edge(run.run_id, (upstream_id, downstream_id))?;
        }

        // only enqueue default tasks with no upstream dependencies
        for task in default_tasks {
            if self.get_task_depth(run.run_id, task.id)? == 0 {
                self.enqueue_task(
                    run.run_id,
                    task.id,
                    run.scheduled_date_for_run,
                    run.pipeline_name.to_string(),
                    false,
                )?;
            }
        }

        Ok(())
    }

    fn handle_task_result(
        &mut self,
        run_id: usize,
        queued_task: &QueuedTask,
        result: TaskResult,
    ) -> Result<()> {
        // TODO check if this result has been handled, ignore handling if so

        let mut result = result;
        let mut branch_left = false;

        if result.is_branch {
            let branch = result.result.as_object().unwrap();
            branch_left = branch.contains_key("Left");

            result.result = if branch_left {
                result.result["Left"].take()
            } else {
                result.result["Right"].take()
            };
        }

        self.insert_task_results(run_id, &result)?;

        result.print_task_result(
            self.get_template_args(run_id, result.task_id)?,
            self.get_log(run_id, result.task_id, result.attempt)?,
        );

        if result.needs_retry() {
            if result.is_sensor {
                println!(
                    "\nsensor attempt failed, retrying #{}\n",
                    result.attempt + 1
                );
            } else {
                println!(
                    "\nattempt failed, retrying {}/{}\n",
                    result.attempt + 1,
                    result.max_attempts
                );
            }
            self.set_task_status(run_id, result.task_id, TaskStatus::RetryPending)?;
            self.enqueue_task(
                run_id,
                result.task_id,
                queued_task.scheduled_date_for_run,
                queued_task.pipeline_name.clone(),
                false,
            )?;
            return Ok(());
        }

        if result.is_branch && result.success {
            let skip_task = if branch_left {
                result.task_id + 2
            } else {
                result.task_id + 1
            };
            let mut to_skip = vec![skip_task];
            to_skip.append(&mut self.get_downstream(run_id, skip_task)?);

            while let Some(curr) = to_skip.pop() {
                to_skip.append(&mut self.get_downstream(run_id, curr)?);
                self.set_task_status(run_id, curr, TaskStatus::Skipped)?;
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
        )?;

        if !result.premature_failure && self.task_needs_running(run_id, result.task_id)? {
            self.enqueue_task(
                run_id,
                result.task_id,
                queued_task.scheduled_date_for_run,
                queued_task.pipeline_name.clone(),
                false,
            )?;
        } else {
            for downstream in self.get_downstream(run_id, result.task_id)? {
                if !self.is_task_done(run_id, downstream)?
                    && self.trigger_rules_satisfied(run_id, downstream)?
                {
                    self.enqueue_task(
                        run_id,
                        downstream,
                        queued_task.scheduled_date_for_run,
                        queued_task.pipeline_name.clone(),
                        false,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn run_task<D: AsRef<OsStr>>(
        &mut self,
        run_id: usize,
        task: &Task,
        attempt: usize,
        resolution_result: &Value,
        tpt_path: D,
        scheduled_date_for_run: DateTime<Utc>,
    ) -> Result<TaskResult> {
        if task.lazy_expand {
            let downstream = self.get_downstream(run_id, task.id)?;

            let mut lazy_ids = vec![];
            for res in resolution_result.as_array().unwrap() {
                let new_id = self.append_new_task_and_set_status_to_pending(
                    run_id,
                    &task.name,
                    &task.function,
                    res,
                    &task.options,
                    false,
                    true,
                    false,
                    false,
                )?;

                lazy_ids.push(new_id);
                self.insert_edge(run_id, (task.id, new_id))?;
            }

            if !downstream.is_empty() {
                let function_name = function_name_as_string(&collector).to_string();

                let collector_id = self.append_new_task_and_set_status_to_pending(
                    run_id,
                    &task.name,
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
                    false,
                )?;
                self.update_referenced_dependencies(run_id, collector_id)?;

                for lazy_id in &lazy_ids {
                    self.insert_edge(run_id, (*lazy_id, collector_id))?;
                }

                for d in &downstream {
                    self.insert_edge(run_id, (collector_id, *d))?;

                    self.set_template_args(
                        run_id,
                        *d,
                        &serde_json::to_string(&self.get_template_args(run_id, *d)?)
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
                    )?;
                }
                for d in &downstream {
                    self.remove_edge(run_id, (task.id, *d))?;
                    self.update_referenced_dependencies(run_id, *d)?;
                    self.delete_task_depth(run_id, *d)?;
                    self.enqueue_task(
                        run_id,
                        *d,
                        scheduled_date_for_run,
                        self.get_pipeline_name()?,
                        true,
                    )?;
                }

                self.enqueue_task(
                    run_id,
                    collector_id,
                    scheduled_date_for_run,
                    self.get_pipeline_name()?,
                    true,
                )?;
            }
            for lazy_id in &lazy_ids {
                self.enqueue_task(
                    run_id,
                    *lazy_id,
                    scheduled_date_for_run,
                    self.get_pipeline_name()?,
                    true,
                )?;
            }

            return Ok(TaskResult {
                task_id: task.id,
                result: Value::Null,
                attempt,
                max_attempts: task.options.max_attempts,
                name: task.name.clone(),
                function: task.function.clone(),
                success: true,
                resolved_args_str: "".into(),
                started: None,
                ended: None,
                elapsed: 0,
                premature_failure: false,
                premature_failure_error_str: "".into(),
                is_branch: task.is_branch,
                is_sensor: task.options.is_sensor,
                exit_code: None,
            });
        }

        task.execute(
            resolution_result,
            attempt,
            self.get_log_handle_closure(run_id, task.id, attempt)?,
            self.get_log_handle_closure(run_id, task.id, attempt)?,
            self.take_last_stdout_line(run_id, task.id, attempt)?,
            self.get_pipeline_path()?,
            tpt_path,
            run_id,
        )
    }

    fn resolve_args(
        &mut self,
        run_id: usize,
        template_args: &Value,
        upstream_deps: &HashMap<(usize, String), String>,
    ) -> Result<Value> {
        let mut results: HashMap<usize, Value> = HashMap::new();
        for ((upstream_id, _), key) in upstream_deps {
            if results.contains_key(upstream_id) {
                continue;
            }

            // if !self.is_task_completed(run_id, *upstream_task_id) {
            //     return Err(Error::new(
            //         ErrorKind::NotFound,
            //         format!("upstream task_id {} does not exist!", upstream_task_id),
            //     ));
            // }
            let task_result = self.get_task_result(run_id, *upstream_id)?;
            results.insert(*upstream_id, task_result.result.clone());
            if !task_result.success {
                return Err(anyhow::Error::msg(format!(
                    "upstream task_id {} failed!",
                    upstream_id
                )));
            }

            if key.is_empty() {
                continue;
            }

            if task_result.result.is_object() {
                let upstream_results_map = task_result.result.as_object().unwrap();

                if !upstream_results_map.contains_key(key) {
                    return Err(anyhow::Error::msg(format!(
                        "upstream task_id {} result does not contain key {}",
                        upstream_id, key
                    )));
                }

                continue;
            }

            return Err(anyhow::Error::msg(format!(
                "upstream_task_id {} result type '{:?}' is not a map",
                upstream_id, task_result.result
            )));
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

                    if !map.contains_key(UPSTREAM_TASK_ID_KEY) {
                        return a.clone();
                    }

                    let upstream_id = map[UPSTREAM_TASK_ID_KEY].as_u64().unwrap() as usize;
                    let result: Value = results[&upstream_id].clone();
                    if map.contains_key(UPSTREAM_TASK_RESULT_KEY) {
                        result.as_object().unwrap()[map[UPSTREAM_TASK_RESULT_KEY].as_str().unwrap()]
                            .clone()
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

            for ((upstream_id, original_key), key) in upstream_deps {
                let result = &results[upstream_id];

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

    fn work<D: AsRef<OsStr>>(
        &mut self,
        temp_queued_task: &TempQueuedTask,
        tpt_path: D,
    ) -> Result<()> {
        let task = self.get_task_by_id(
            temp_queued_task.queued_task.run_id,
            temp_queued_task.queued_task.task_id,
        )?;
        let dependency_keys =
            self.get_dependencies(temp_queued_task.queued_task.run_id, task.id)?;
        let result = match self.resolve_args(
            temp_queued_task.queued_task.run_id,
            &task.template_args,
            &dependency_keys,
        ) {
            Ok(resolution_result) => self.run_task(
                temp_queued_task.queued_task.run_id,
                &task,
                temp_queued_task.queued_task.attempt,
                &resolution_result,
                tpt_path,
                temp_queued_task.queued_task.scheduled_date_for_run,
            )?,
            Err(resolution_result) => TaskResult::premature_error(
                task.id,
                temp_queued_task.queued_task.attempt,
                task.options.max_attempts,
                task.name,
                task.function,
                resolution_result.to_string(),
                task.is_branch,
                task.options.is_sensor,
                None,
                None,
            ),
        };
        self.handle_task_result(
            temp_queued_task.queued_task.run_id,
            &temp_queued_task.queued_task,
            result,
        )?;
        Ok(())
    }

    fn update_referenced_dependencies(
        &mut self,
        run_id: usize,
        downstream_id: usize,
    ) -> Result<()> {
        let template_args = self.get_template_args(run_id, downstream_id)?;

        if template_args.is_array() {
            for value in template_args
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| v.is_object())
            {
                let map_value = value.as_object().unwrap();
                if map_value.contains_key(UPSTREAM_TASK_ID_KEY) {
                    let upstream_id = map_value
                        .get(UPSTREAM_TASK_ID_KEY)
                        .unwrap()
                        .as_u64()
                        .unwrap() as usize;

                    self.set_dependency(
                        run_id,
                        downstream_id,
                        (upstream_id, "".into()),
                        if map_value.contains_key(UPSTREAM_TASK_RESULT_KEY) {
                            map_value
                                .get(UPSTREAM_TASK_RESULT_KEY)
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string()
                        } else {
                            "".into()
                        },
                    )?;
                    self.insert_edge(run_id, (upstream_id, downstream_id))?;
                }
            }
        } else if template_args.is_object() {
            let template_args = template_args.as_object().unwrap();
            if template_args.contains_key(UPSTREAM_TASK_ID_KEY) {
                let upstream_id = template_args
                    .get(UPSTREAM_TASK_ID_KEY)
                    .unwrap()
                    .as_u64()
                    .unwrap() as usize;
                self.set_dependency(
                    run_id,
                    downstream_id,
                    (upstream_id, "".into()),
                    if template_args.contains_key(UPSTREAM_TASK_RESULT_KEY) {
                        template_args
                            .get(UPSTREAM_TASK_RESULT_KEY)
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_string()
                    } else {
                        "".into()
                    },
                )?;

                self.insert_edge(run_id, (upstream_id, downstream_id))?;

                return Ok(());
            }

            for (key, value) in template_args.iter().filter(|(_, v)| v.is_object()) {
                let map = value.as_object().unwrap();
                if map.contains_key(UPSTREAM_TASK_ID_KEY) {
                    let upstream_id =
                        map.get(UPSTREAM_TASK_ID_KEY).unwrap().as_u64().unwrap() as usize;

                    self.set_dependency(
                        run_id,
                        downstream_id,
                        (upstream_id, key.to_string()),
                        if map.contains_key(UPSTREAM_TASK_RESULT_KEY) {
                            map.get(UPSTREAM_TASK_RESULT_KEY)
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string()
                        } else {
                            "".into()
                        },
                    )?;

                    self.insert_edge(run_id, (upstream_id, downstream_id))?;
                }
            }
        }

        Ok(())
    }
}
