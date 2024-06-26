use deadpool_redis::{redis::cmd, Pool};
use log::debug;
use std::collections::{HashMap, HashSet};
use thepipelinetool_runner::run::Run;
use thepipelinetool_runner::{
    backend::Backend, pipeline::Pipeline, pipeline_options::PipelineOptions,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use thepipelinetool_core::dev::*;
use timed::timed;

const TASK_STATUS_KEY: &str = "ts";
const TASK_RESULTS_KEY: &str = "trs";
const RUNS_KEY: &str = "runs";
const NEXT_RUN_KEY: &str = "nr";
const SCHEDULED_DATES_KEY: &str = "ld";
const DEPTH_KEY: &str = "d";
const TASK_RESULT_KEY: &str = "tr";
const LOG_KEY: &str = "l";
const TASK_ATTEMPT_KEY: &str = "a";
const DEPENDENCY_KEYS_KEY: &str = "dk";
const EDGES_KEY: &str = "e";
const TASKS_KEY: &str = "tks";
const TASK_ID_KEY: &str = "ti";
const TASK_KEY: &str = "t";
const TEMPLATE_ARGS_KEY: &str = "ta";
const DEFAULT_TASKS_KEY: &str = "dt";
const DEFAULT_EDGES_KEY: &str = "de";
const DEFAULT_OPTIONS_KEY: &str = "do";
const PIPELINES_KEY: &str = "p";
const PIPELINE_PATH_KEY: &str = "pp";

macro_rules! block_on {
    // Textual definition.
    ($body:block) => {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(async { $body }))
    };
}

#[derive(Clone)]
pub struct RedisBackend {
    name: Option<String>,
    pool: Pool,
}

impl RedisBackend {
    pub fn dummy(pool: Pool) -> Self {
        Self { name: None, pool }
    }

    pub fn from(pipeline_name: &str, pool: Pool) -> Self {
        Self {
            name: Some(pipeline_name.to_string()),
            pool,
        }
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_pipelines(pool: Pool) -> Result<HashSet<String>> {
        let mut conn = pool.get().await.expect("DB connection failed");
        Ok(HashSet::from_iter(
            cmd("SMEMBERS")
                .arg(PIPELINES_KEY)
                .query_async::<_, Vec<String>>(&mut conn)
                .await?,
        ))
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn upload_pipeline(
        pipeline: &Pipeline,
        pipeline_name: &str,
        pool: Pool,
    ) -> Result<()> {
        let mut conn = pool.get().await.expect("DB connection failed");

        cmd("SET")
            .arg(format!("{DEFAULT_OPTIONS_KEY}:{pipeline_name}"))
            .arg(serde_json::to_string(&pipeline.options)?)
            .query_async::<_, String>(&mut conn)
            .await?;

        cmd("SET")
            .arg(format!("{DEFAULT_TASKS_KEY}:{pipeline_name}"))
            .arg(serde_json::to_string(&pipeline.tasks)?)
            .query_async::<_, String>(&mut conn)
            .await?;

        cmd("SET")
            .arg(format!("{DEFAULT_EDGES_KEY}:{pipeline_name}"))
            .arg(serde_json::to_string(&pipeline.edges)?)
            .query_async::<_, String>(&mut conn)
            .await?;

        cmd("SET")
            .arg(format!("{PIPELINE_PATH_KEY}:{pipeline_name}"))
            .arg(pipeline.path.clone())
            .query_async::<_, String>(&mut conn)
            .await?;

        cmd("SADD")
            .arg(PIPELINES_KEY)
            .arg(pipeline_name)
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_options(&self) -> Result<PipelineOptions> {
        let mut conn = self.pool.get().await.expect("DB connection failed");
        let pipeline_name = self.get_pipeline_name()?;
        Ok(serde_json::from_str(
            &cmd("GET")
                .arg(format!("{DEFAULT_OPTIONS_KEY}:{pipeline_name}"))
                .query_async::<_, String>(&mut conn)
                .await?,
        )?)
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_temp_queue(&self) -> Result<Vec<TempQueuedTask>> {
        let mut conn = self.pool.get().await.expect("DB connection failed");

        let members = cmd("SMEMBERS")
            .arg("tmpqueue")
            .query_async::<_, Vec<String>>(&mut conn)
            .await?;

        let mut v = vec![];

        for s in members {
            v.push(serde_json::from_str(&s)?);
        }
        Ok(v)
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_all_results(
        run_id: usize,
        task_id: usize,
        pool: Pool,
    ) -> Result<Vec<TaskResult>> {
        let mut conn = pool.get().await.expect("DB connection failed");
        let members = cmd("LRANGE")
            .arg(format!("{TASK_RESULTS_KEY}:{run_id}:{task_id}"))
            .arg(0)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await?;

        let mut v = vec![];

        for s in members {
            v.push(serde_json::from_str(&s)?);
        }
        Ok(v)
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_runs(pipeline_name: &str, pool: Pool) -> Result<Vec<Run>> {
        let mut conn = pool.get().await.expect("DB connection failed");
        let members = cmd("LRANGE")
            .arg(format!("{RUNS_KEY}:{pipeline_name}"))
            .arg(0)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await?;

        let mut v = vec![];

        for s in members {
            v.push(serde_json::from_str(&s)?);
        }
        Ok(v)
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_last_run(pipeline_name: &str, pool: Pool) -> Result<Option<Run>> {
        let mut conn = pool.get().await.expect("DB connection failed");
        let members = cmd("LRANGE")
            .arg(format!("{RUNS_KEY}:{pipeline_name}"))
            .arg(-1)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await?;
        if members.is_empty() {
            return Ok(None);
        }
        Ok(serde_json::from_str(&members[0])?)
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_next_run(pipeline_name: &str, pool: Pool) -> Result<Option<String>> {
        let mut conn = pool.get().await.expect("DB connection failed");
        let members = cmd("GET")
            .arg(format!("{NEXT_RUN_KEY}:{pipeline_name}"))
            .query_async::<_, String>(&mut conn)
            .await;
        if let Ok(string) = members {
            Ok(Some(string))
        } else {
            Err(anyhow!(format!(
                "could not find next run for pipeline '{}'",
                pipeline_name
            )))
        }
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn set_next_run(
        pipeline_name: &str,
        scheduled_date: Option<DateTime<Utc>>,
        pool: Pool,
    ) -> Result<()> {
        let mut conn = pool.get().await.expect("DB connection failed");
        cmd("SET")
            .arg(format!("{NEXT_RUN_KEY}:{pipeline_name}"))
            .arg(serde_json::to_string(&scheduled_date)?)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_recent_runs(pipeline_name: &str, pool: Pool) -> Result<Vec<Run>> {
        let mut conn = pool.get().await.expect("DB connection failed");
        let members = cmd("LRANGE")
            .arg(format!("{RUNS_KEY}:{pipeline_name}"))
            .arg(-10)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await?;

        let mut v = vec![];

        for s in members {
            v.push(serde_json::from_str(&s)?);
        }
        Ok(v)
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn contains_scheduled_date(
        pipeline_name: &str,
        scheduled_date_for_run: DateTime<Utc>,
        pool: Pool,
    ) -> Result<bool> {
        let mut conn = pool.get().await.expect("DB connection failed");
        Ok(cmd("SISMEMBER")
            .arg(format!("{SCHEDULED_DATES_KEY}:{pipeline_name}"))
            .arg(scheduled_date_for_run.to_string())
            .query_async::<_, bool>(&mut conn)
            .await?)
    }

    // #[timed(duration(printer = "debug!"))]
    pub async fn get_running_tasks_count(&self) -> Result<usize> {
        let mut conn = self.pool.get().await.expect("DB connection failed");
        Ok(cmd("SCARD")
            .arg("tmpqueue")
            .query_async::<_, usize>(&mut conn)
            .await?)
    }
}

impl Backend for RedisBackend {
    #[timed(duration(printer = "debug!"))]
    fn get_queue_length(&self) -> Result<usize> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            Ok(cmd("ZCOUNT")
                .arg("queue")
                .arg(i32::MIN)
                .arg(i32::MAX)
                .query_async::<_, usize>(&mut conn)
                .await?)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn remove_from_temp_queue(&self, temp_queued_task: &TempQueuedTask) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            cmd("SREM")
                .arg("tmpqueue")
                .arg(serde_json::to_string(&temp_queued_task)?)
                .query_async::<_, usize>(&mut conn)
                .await?;
            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn delete_task_depth(&mut self, run_id: usize, task_id: usize) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            cmd("DEL")
                .arg(format!("{DEPTH_KEY}:{run_id}:{task_id}"))
                .query_async::<_, usize>(&mut conn)
                .await?;

            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_log(&mut self, run_id: usize, task_id: usize, attempt: usize) -> Result<String> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let members = cmd("LRANGE")
                .arg(format!("{LOG_KEY}:{run_id}:{task_id}:{attempt}"))
                .arg(0)
                .arg(-1)
                .query_async::<_, Vec<String>>(&mut conn)
                .await?;

            Ok(members.join("\n"))
        })
    }

    // #[timed(duration(printer = "debug!"))]
    fn get_log_handle_closure(
        &mut self,
        run_id: usize,
        task_id: usize,
        attempt: usize,
    ) -> Result<Box<dyn Fn(String) -> Result<()> + Send>> {
        let pool = self.pool.clone();
        Ok(Box::new(move |s| {
            tokio::runtime::Runtime::new()?.block_on(async {
                let mut conn = pool.get().await.expect("DB connection failed");
                cmd("RPUSH")
                    .arg(format!("{LOG_KEY}:{run_id}:{task_id}:{attempt}"))
                    .arg(s)
                    .query_async::<_, usize>(&mut conn)
                    .await?;

                Ok(())
            })
        }))
    }

    #[timed(duration(printer = "debug!"))]
    fn get_task_result(&mut self, run_id: usize, task_id: usize) -> Result<TaskResult> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            Ok(serde_json::from_str(
                &cmd("GET")
                    .arg(format!("{TASK_RESULT_KEY}:{run_id}:{task_id}"))
                    .query_async::<_, String>(&mut conn)
                    .await?,
            )?)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_attempt_by_task_id(
        &self,
        run_id: usize,
        task_id: usize,
        is_dynamic: bool,
    ) -> Result<usize> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            Ok(cmd("INCR")
                .arg(format!(
                    "{TASK_ATTEMPT_KEY}:{run_id}:{task_id}:{is_dynamic}"
                ))
                .query_async::<_, usize>(&mut conn)
                .await?)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_task_status(&self, run_id: usize, task_id: usize) -> Result<TaskStatus> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            Ok(serde_json::from_str(
                &cmd("GET")
                    .arg(format!("{TASK_STATUS_KEY}:{run_id}:{task_id}"))
                    .query_async::<_, String>(&mut conn)
                    .await?,
            )?)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn set_task_status(
        &mut self,
        run_id: usize,
        task_id: usize,
        task_status: TaskStatus,
    ) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            cmd("SET")
                .arg(format!("{TASK_STATUS_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&task_status)?)
                .query_async::<_, String>(&mut conn)
                .await?;

            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn create_new_run(&mut self, scheduled_date_for_run: DateTime<Utc>) -> Result<Run> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            let run_id = cmd("INCR")
                .arg("run")
                .query_async::<_, usize>(&mut conn)
                .await?
                - 1;

            let pipeline_name = self.get_pipeline_name()?;
            let run = Run {
                run_id,
                pipeline_name: pipeline_name.to_string(),
                scheduled_date_for_run,
            };

            cmd("RPUSH")
                .arg(format!("{RUNS_KEY}:{pipeline_name}"))
                .arg(serde_json::to_string(&run)?)
                .query_async::<_, ()>(&mut conn)
                .await?;

            Ok(run)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn insert_task_results(&mut self, run_id: usize, result: &TaskResult) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let res = serde_json::to_string(result)?;
            let task_id = result.task_id;

            cmd("RPUSH")
                .arg(format!("{TASK_RESULTS_KEY}:{run_id}:{task_id}"))
                .arg(&res)
                .query_async::<_, ()>(&mut conn)
                .await?;
            cmd("SET")
                .arg(format!("{TASK_RESULT_KEY}:{run_id}:{task_id}"))
                .arg(res)
                .query_async::<_, ()>(&mut conn)
                .await?;

            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_dependencies(
        &mut self,
        run_id: usize,
        task_id: usize,
    ) -> Result<HashMap<(usize, String), String>> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            let members = cmd("SMEMBERS")
                .arg(format!("{DEPENDENCY_KEYS_KEY}:{run_id}:{task_id}"))
                .query_async::<_, Vec<String>>(&mut conn)
                .await?;

            let mut k: Vec<((usize, String), String)> = vec![];

            for v in members {
                k.push(serde_json::from_str(&v)?);
            }
            Ok(HashMap::from_iter(k.into_iter()))
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn set_dependency(
        &mut self,
        run_id: usize,
        task_id: usize,
        upstream: (usize, String),
        v: String,
    ) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            cmd("SADD")
                .arg(format!("{DEPENDENCY_KEYS_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&(upstream, v))?)
                .query_async::<_, ()>(&mut conn)
                .await?;

            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_downstream(&self, run_id: usize, task_id: usize) -> Result<Vec<usize>> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let members = cmd("SMEMBERS")
                .arg(format!("{EDGES_KEY}:{run_id}"))
                .query_async::<_, Vec<String>>(&mut conn)
                .await?;
            let mut downstream = vec![];
            for f in members {
                let (up, down): (usize, usize) = serde_json::from_str(&f)?;
                if up == task_id {
                    downstream.push(down)
                }
            }
            Ok(downstream)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_upstream(&self, run_id: usize, task_id: usize) -> Result<Vec<usize>> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let members = cmd("SMEMBERS")
                .arg(&[format!("{EDGES_KEY}:{run_id}")])
                .query_async::<_, Vec<String>>(&mut conn)
                .await?;

            let mut upstream = vec![];
            for f in members {
                let (up, down): (usize, usize) = serde_json::from_str(&f)?;
                if down == task_id {
                    upstream.push(up)
                }
            }
            Ok(upstream)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn remove_edge(&mut self, run_id: usize, edge: (usize, usize)) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            cmd("SREM")
                .arg(format!("{EDGES_KEY}:{run_id}"))
                .arg(serde_json::to_string(&edge)?)
                .query_async::<_, usize>(&mut conn)
                .await?;
            let keys_to_remove: Vec<((usize, String), String)> = self
                .get_dependencies(run_id, edge.1)?
                .iter()
                .filter_map(|((upstream, key), val)| {
                    if upstream == &edge.0 {
                        Some(((*upstream, key.to_string()), val.to_string()))
                    } else {
                        None
                    }
                })
                .collect();
            for h in &keys_to_remove {
                cmd("SREM")
                    .arg(format!("{DEPENDENCY_KEYS_KEY}:{run_id}:{}", edge.1))
                    .arg(serde_json::to_string(&h)?)
                    .query_async::<_, ()>(&mut conn)
                    .await?;
            }

            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn insert_edge(&mut self, run_id: usize, edge: (usize, usize)) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            cmd("SADD")
                .arg(format!("{EDGES_KEY}:{run_id}"))
                .arg(serde_json::to_string(&edge)?)
                .query_async::<_, ()>(&mut conn)
                .await?;

            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_all_tasks(&self, run_id: usize) -> Result<Vec<Task>> {
        block_on!({
            let mut conn: deadpool_redis::Connection =
                self.pool.get().await.expect("DB connection failed");
            let members = cmd("SMEMBERS")
                .arg(format!("{TASKS_KEY}:{run_id}"))
                .query_async::<_, Vec<String>>(&mut conn)
                .await?;

            let mut tasks = vec![];

            for m in members {
                tasks.push(serde_json::from_str(&m)?)
            }

            Ok(tasks)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_default_tasks(&self) -> Result<Vec<Task>> {
        block_on!({
            let pipeline_name = self.get_pipeline_name()?;
            dbg!(&pipeline_name);
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let members = cmd("GET")
                .arg(format!("{DEFAULT_TASKS_KEY}:{pipeline_name}"))
                .query_async::<_, String>(&mut conn)
                .await?;

            Ok(serde_json::from_str(&members)?)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_default_edges(&self) -> Result<HashSet<(usize, usize)>> {
        block_on!({
            let pipeline_name = self.get_pipeline_name()?;
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let members = cmd("GET")
                .arg(format!("{DEFAULT_EDGES_KEY}:{pipeline_name}"))
                .query_async::<_, String>(&mut conn)
                .await?;

            Ok(serde_json::from_str(&members)?)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_task_by_id(&self, run_id: usize, task_id: usize) -> Result<Task> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            Ok(serde_json::from_str(
                &cmd("GET")
                    .arg(format!("{TASK_KEY}:{run_id}:{task_id}"))
                    .query_async::<_, String>(&mut conn)
                    .await?,
            )?)
        })
    }

    #[timed(duration(printer = "debug!"))]
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
    ) -> Result<usize> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            let task_id = cmd("INCR")
                .arg(format!("{TASK_ID_KEY}:{run_id}"))
                .query_async::<_, usize>(&mut conn)
                .await?
                - 1;

            let task = Task {
                id: task_id,
                name: name.to_owned(),
                function: function_name.to_owned(),
                template_args: template_args.to_owned(),
                options: options.to_owned(),
                lazy_expand,
                is_dynamic,
                is_branch,
                use_trigger_params,
            };
            cmd("SADD")
                .arg(format!("{TASKS_KEY}:{run_id}"))
                .arg(serde_json::to_string(&task)?)
                .query_async::<_, usize>(&mut conn)
                .await?;
            cmd("SET")
                .arg(format!("{TASK_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&task)?)
                .query_async::<_, ()>(&mut conn)
                .await?;
            cmd("SET")
                .arg(format!("{TEMPLATE_ARGS_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&task.template_args)?)
                .query_async::<_, ()>(&mut conn)
                .await?;
            self.set_task_status(run_id, task_id, TaskStatus::Pending)?;
            Ok(task_id)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_template_args(&self, run_id: usize, task_id: usize) -> Result<Value> {
        let task = self.get_task_by_id(run_id, task_id)?;
        Ok(task.template_args)
    }

    #[timed(duration(printer = "debug!"))]
    fn set_template_args(
        &mut self,
        run_id: usize,
        task_id: usize,
        template_args_str: &str,
    ) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let mut task = self.get_task_by_id(run_id, task_id)?;
            task.template_args = serde_json::from_str(template_args_str)?;

            cmd("SET")
                .arg(format!("{TASK_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&task)?)
                .query_async::<_, String>(&mut conn)
                .await?;

            Ok(())
        })
    }

    // #[timed(duration(printer = "debug!"))]
    fn pop_priority_queue(&mut self) -> Result<Option<TempQueuedTask>> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            let res = cmd("ZPOPMIN")
                .arg(&["queue".to_string(), "1".to_string()]) // TODO timeout arg
                .query_async::<_, Vec<String>>(&mut conn)
                .await;

            if let Ok(vec) = &res {
                if !vec.is_empty() {
                    let temp_queued_task = TempQueuedTask {
                        popped_date: Utc::now(),
                        queued_task: serde_json::from_str(&vec[0])?,
                    };
                    cmd("SADD")
                        .arg(&[
                            "tmpqueue".to_string(),
                            serde_json::to_string(&temp_queued_task)?,
                        ])
                        .query_async::<_, ()>(&mut conn)
                        .await?;
                    return Ok(Some(temp_queued_task));
                }
            } else {
                println!("{:#?}", res.unwrap_err().detail());
            }

            Ok(None)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_task_depth(&mut self, run_id: usize, task_id: usize) -> Result<usize> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            if !cmd("EXISTS")
                .arg(format!("{DEPTH_KEY}:{run_id}:{task_id}"))
                .query_async::<_, bool>(&mut conn)
                .await?
            {
                let mut max_depth = 0;

                for upstream_id in self.get_upstream(run_id, task_id)? {
                    let new_depth = self.get_task_depth(run_id, upstream_id)? + 1;
                    if new_depth > max_depth {
                        max_depth = new_depth;
                    }
                }
                self.set_task_depth(run_id, task_id, max_depth)?;
            }
            Ok(cmd("GET")
                .arg(format!("{DEPTH_KEY}:{run_id}:{task_id}"))
                .query_async::<_, usize>(&mut conn)
                .await?)
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn set_task_depth(&mut self, run_id: usize, task_id: usize, depth: usize) -> Result<()> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");

            cmd("SET")
                .arg(&[format!("{DEPTH_KEY}:{run_id}:{task_id}"), depth.to_string()])
                .query_async::<_, ()>(&mut conn)
                .await?;

            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn enqueue_task(
        &mut self,
        run_id: usize,
        task_id: usize,
        scheduled_date_for_run: DateTime<Utc>,
        pipeline_name: String,
        is_dynamic: bool,
    ) -> Result<()> {
        block_on!({
            let depth = self.get_task_depth(run_id, task_id)?;
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let attempt: usize = self.get_attempt_by_task_id(run_id, task_id, is_dynamic)?;

            // remove previous attempts (this is needed for lazy expand)
            let members = cmd("ZRANGEBYSCORE")
                .arg("queue")
                .arg("-inf")
                .arg("+inf")
                .query_async::<_, Vec<String>>(&mut conn)
                .await?;
            for m in members {
                let queued_task: QueuedTask = serde_json::from_str(&m)?;
                if queued_task.run_id == run_id && queued_task.task_id == task_id {
                    cmd("ZREM")
                        .arg(&["queue".to_string(), serde_json::to_string(&queued_task)?])
                        .query_async::<_, usize>(&mut conn)
                        .await?;
                }
            }
            cmd("ZADD")
                .arg(&[
                    "queue".to_string(),
                    depth.to_string(),
                    serde_json::to_string(&QueuedTask {
                        task_id,
                        run_id,
                        pipeline_name,
                        scheduled_date_for_run,
                        attempt,
                    })?,
                ])
                .query_async::<_, usize>(&mut conn)
                .await?;
            Ok(())
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn print_priority_queue(&mut self) -> Result<()> {
        Ok(())
    }

    // #[timed(duration(printer = "debug!"))]
    fn take_last_stdout_line(
        &mut self,
        run_id: usize,
        task_id: usize,
        attempt: usize,
    ) -> Result<Box<dyn Fn() -> Result<String> + Send>> {
        let pool = self.pool.clone();
        Ok(Box::new(move || {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut conn = pool.get().await.expect("DB connection failed");
                    let lines = cmd("RPOP")
                        .arg(format!("{LOG_KEY}:{run_id}:{task_id}:{attempt}"))
                        .arg(1)
                        .query_async::<_, Vec<String>>(&mut conn)
                        .await?;

                    if lines.is_empty() {
                        Ok("".to_string())
                    } else {
                        Ok(lines[0].to_string())
                    }
                })
            })
        }))
    }

    fn get_pipeline_name(&self) -> Result<String> {
        if let Some(name) = &self.name {
            Ok(name.into())
        } else {
            Err(anyhow!("no name found"))
        }
    }

    fn get_pipeline_path(&self) -> Result<String> {
        block_on!({
            let mut conn = self.pool.get().await.expect("DB connection failed");
            let pipeline_name = self.get_pipeline_name()?;

            Ok(cmd("GET")
                .arg(format!("{PIPELINE_PATH_KEY}:{pipeline_name}"))
                .query_async::<_, String>(&mut conn)
                .await?)
        })
    }
}
