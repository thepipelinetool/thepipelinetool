use deadpool_redis::{redis::cmd, Pool};
use log::debug;
use std::{
    collections::{HashMap, HashSet},
    process::Command,
};
use thepipelinetool_runner::{backend::Backend, blanket_backend::BlanketBackend, Runner};

use chrono::{DateTime, Utc};
use std::str::FromStr;
use thepipelinetool_core::dev::*;
use timed::timed;

const TASK_STATUS_KEY: &str = "ts";
const TASK_RESULTS_KEY: &str = "trs";
const RUNS_KEY: &str = "runs";
const LOGICAL_DATES_KEY: &str = "ld";
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

macro_rules! block_on {
    // Textual definition.
    ($body:block) => {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(async { $body }))
    };
}

impl<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> Runner<U> for LocalRunner<U> {
    fn run(&mut self, ordered_queued_task: &OrderedQueuedTask) {
        let mut cmd = Command::new(&self.executor_path);
        cmd.arg(serde_json::to_string(ordered_queued_task).unwrap());
        let _ = spawn(
            cmd,
            Box::new(|x| print!("{x}")),
            Box::new(|x| eprint!("{x}")),
        );
    }

    fn get_max_parallelism(&self) -> usize {
        self.max_parallelism
    }

    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask> {
        self.backend.pop_priority_queue()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Run {
    pub run_id: usize,
    pub scheduled_date_for_dag_run: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RedisBackend {
    edges: Option<HashSet<(usize, usize)>>,
    nodes: Option<Vec<Task>>,
    // name: String,
    pool: Pool,
}

impl RedisBackend {
    #[timed(duration(printer = "debug!"))]
    pub fn dummy(pool: Pool) -> Self {
        Self {
            // name: "".into(),
            edges: None,
            nodes: None,
            pool,
        }
    }

    // #[timed(duration(printer = "debug!"))]
    pub fn from(nodes: Vec<Task>, edges: HashSet<(usize, usize)>, pool: Pool) -> Self {
        // let nodes = _get_default_tasks(name);
        // let edges = _get_default_edges(name);

        Self {
            // name: name.into(),
            edges: Some(edges),
            nodes: Some(nodes),
            pool,
        }
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_temp_queue(&self) -> Vec<QueuedTask> {
        let mut conn = self.pool.get().await.unwrap();

        cmd("SMEMBERS")
            .arg("tmpqueue") // TODO timeout arg
            .query_async::<_, Vec<String>>(&mut conn)
            .await
            .unwrap()
            .iter()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_all_results(run_id: usize, task_id: usize, pool: Pool) -> Vec<TaskResult> {
        let mut conn = pool.get().await.unwrap();
        cmd("LRANGE")
            .arg(format!("{TASK_RESULTS_KEY}:{run_id}:{task_id}"))
            .arg(0)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await
            .unwrap()
            .iter()
            .map(|v| serde_json::from_str(v).unwrap())
            .collect()
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_runs(dag_name: &str, pool: Pool) -> Vec<Run> {
        let mut conn = pool.get().await.unwrap();
        cmd("LRANGE")
            .arg(format!("{RUNS_KEY}:{dag_name}"))
            .arg(0)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await
            .unwrap()
            .iter()
            .map(|v| serde_json::from_str(v).unwrap())
            .collect()
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn get_last_run(dag_name: &str, pool: Pool) -> Option<Run> {
        let mut conn = pool.get().await.unwrap();
        cmd("LRANGE")
            .arg(format!("{RUNS_KEY}:{dag_name}"))
            .arg(-1)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await
            .unwrap_or_default()
            .first()
            .map(|run| serde_json::from_str(run).unwrap())
    }

    // #[timed(duration(printer = "debug!"))]
    pub async fn get_recent_runs(dag_name: &str, pool: Pool) -> Vec<Run> {
        let mut conn = pool.get().await.unwrap();
        cmd("LRANGE")
            .arg(format!("{RUNS_KEY}:{dag_name}"))
            .arg(-10)
            .arg(-1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await
            .unwrap_or_default()
            .iter()
            .map(|run| serde_json::from_str(run).unwrap())
            .collect()
    }

    #[timed(duration(printer = "debug!"))]
    pub async fn contains_logical_date(
        dag_name: &str,
        dag_hash: &str,
        scheduled_date_for_dag_run: DateTime<Utc>,
        pool: Pool,
    ) -> bool {
        let mut conn = pool.get().await.unwrap();
        cmd("SISMEMBER")
            .arg(format!("{LOGICAL_DATES_KEY}:{dag_name}:{dag_hash}"))
            .arg(scheduled_date_for_dag_run.to_string())
            .query_async::<_, bool>(&mut conn)
            .await
            .unwrap()
    }

    pub async fn get_running_tasks_count(&self) -> usize {
        let mut conn = self.pool.get().await.unwrap();
        cmd("SCARD")
            .arg("tmpqueue")
            .query_async::<_, usize>(&mut conn)
            .await
            .unwrap()
    }
}

impl Backend for RedisBackend {
    // fn load_from_name(&mut self, dag_name: &str) {
    //     self.nodes = _get_default_tasks(dag_name);
    //     self.edges = _get_default_edges(dag_name);
    // }

    fn get_queue_length(&self) -> usize {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            cmd("ZCOUNT")
                .arg("queue")
                .arg(i32::MIN)
                .arg(i32::MAX)
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap()
        })
    }

    fn remove_from_temp_queue(&self, queued_task: &QueuedTask) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("SREM")
                .arg("tmpqueue")
                .arg(serde_json::to_string(queued_task).unwrap())
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
        })
    }

    fn delete_task_depth(&mut self, run_id: usize, task_id: usize) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            cmd("DEL")
                .arg(format!("{DEPTH_KEY}:{run_id}:{task_id}"))
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap();
        });
    }

    #[timed(duration(printer = "debug!"))]
    fn get_log(&mut self, run_id: usize, task_id: usize, attempt: usize) -> String {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("LRANGE")
                .arg(format!("{LOG_KEY}:{run_id}:{task_id}:{attempt}"))
                .arg(0)
                .arg(-1)
                .query_async::<_, Vec<String>>(&mut conn)
                .await
                .unwrap_or_default()
                .join("\n")
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_log_handle_closure(
        &mut self,
        run_id: usize,
        task_id: usize,
        attempt: usize,
    ) -> Box<dyn Fn(String) + Send> {
        let pool = self.pool.clone();
        Box::new(move |s| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut conn = pool.get().await.unwrap();
                cmd("RPUSH")
                    .arg(format!("{LOG_KEY}:{run_id}:{task_id}:{attempt}"))
                    .arg(s)
                    .query_async::<_, usize>(&mut conn)
                    .await
                    .unwrap()
            });
        })
    }

    // #[timed(duration(printer = "debug!"))]
    // fn get_dag_name(&self) -> String {
    //     self.name.clone()
    // }

    #[timed(duration(printer = "debug!"))]
    fn get_task_result(&mut self, run_id: usize, task_id: usize) -> TaskResult {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            serde_json::from_str(
                &cmd("GET")
                    .arg(format!("{TASK_RESULT_KEY}:{run_id}:{task_id}"))
                    .query_async::<_, String>(&mut conn)
                    .await
                    .unwrap(),
            )
            .unwrap()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_attempt_by_task_id(&self, run_id: usize, task_id: usize) -> usize {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            cmd("INCR")
                .arg(format!("{TASK_ATTEMPT_KEY}:{run_id}:{task_id}"))
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_task_status(&mut self, run_id: usize, task_id: usize) -> TaskStatus {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            TaskStatus::from_str(
                &cmd("GET")
                    .arg(format!("{TASK_STATUS_KEY}:{run_id}:{task_id}"))
                    .query_async::<_, String>(&mut conn)
                    .await
                    .unwrap(),
            )
            .unwrap()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn set_task_status(&mut self, run_id: usize, task_id: usize, task_status: TaskStatus) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("SET")
                .arg(format!("{TASK_STATUS_KEY}:{run_id}:{task_id}"))
                .arg(task_status.as_str())
                .query_async::<_, String>(&mut conn)
                .await
                .unwrap();
        });
    }

    #[timed(duration(printer = "debug!"))]
    fn create_new_run(
        &mut self,
        dag_name: &str,
        _dag_hash: &str, // TODO
        scheduled_date_for_dag_run: DateTime<Utc>,
    ) -> usize {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            let run_id = cmd("INCR")
                .arg("run")
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap();

            cmd("RPUSH")
                .arg(format!("{RUNS_KEY}:{dag_name}"))
                .arg(
                    serde_json::to_string(&Run {
                        run_id,
                        scheduled_date_for_dag_run,
                    })
                    .unwrap(),
                )
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
            run_id
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn insert_task_results(&mut self, run_id: usize, result: &TaskResult) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            let res = serde_json::to_string(result).unwrap();
            let task_id = result.task_id;

            cmd("RPUSH")
                .arg(format!("{TASK_RESULTS_KEY}:{run_id}:{task_id}"))
                .arg(&res)
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
            cmd("SET")
                .arg(format!("{TASK_RESULT_KEY}:{run_id}:{task_id}"))
                .arg(res)
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
        });
    }

    #[timed(duration(printer = "debug!"))]
    fn get_dependency_keys(
        &mut self,
        run_id: usize,
        task_id: usize,
    ) -> HashMap<(usize, String), String> {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            let k: Vec<((usize, String), String)> = cmd("SMEMBERS")
                .arg(format!("{DEPENDENCY_KEYS_KEY}:{run_id}:{task_id}"))
                .query_async::<_, Vec<String>>(&mut conn)
                .await
                .unwrap_or_default()
                .iter()
                .map(|v| serde_json::from_str(v).unwrap())
                .collect();
            k.into_iter().collect()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn set_dependency_keys(
        &mut self,
        run_id: usize,
        task_id: usize,
        upstream: (usize, String),
        v: String,
    ) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("SADD")
                .arg(format!("{DEPENDENCY_KEYS_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&(upstream, v)).unwrap())
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_downstream(&self, run_id: usize, task_id: usize) -> Vec<usize> {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("SMEMBERS")
                .arg(format!("{EDGES_KEY}:{run_id}"))
                .query_async::<_, Vec<String>>(&mut conn)
                .await
                .unwrap()
                .iter()
                .map(|f| serde_json::from_str::<(usize, usize)>(f).unwrap())
                .filter_map(|(up, down)| if up == task_id { Some(down) } else { None })
                .collect()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_upstream(&self, run_id: usize, task_id: usize) -> Vec<usize> {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("SMEMBERS")
                .arg(&[format!("{EDGES_KEY}:{run_id}")])
                .query_async::<_, Vec<String>>(&mut conn)
                .await
                .unwrap()
                .iter()
                .map(|f| serde_json::from_str::<(usize, usize)>(f).unwrap())
                .filter_map(|(up, down)| if down == task_id { Some(up) } else { None })
                .collect()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn remove_edge(&mut self, run_id: usize, edge: (usize, usize)) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("SREM")
                .arg(format!("{EDGES_KEY}:{run_id}"))
                .arg(serde_json::to_string(&edge).unwrap())
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap();
            cmd("SREM")
                .arg(format!("{DEPENDENCY_KEYS_KEY}:{run_id}:{}", edge.1))
                .arg(serde_json::to_string(&((edge.0, ""), "")).unwrap())
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
        });
    }

    #[timed(duration(printer = "debug!"))]
    fn insert_edge(&mut self, run_id: usize, edge: (usize, usize)) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            cmd("SADD")
                .arg(format!("{EDGES_KEY}:{run_id}"))
                .arg(serde_json::to_string(&edge).unwrap())
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_all_tasks(&self, run_id: usize) -> Vec<Task> {
        block_on!({
            let mut conn: deadpool_redis::Connection = self.pool.get().await.unwrap();
            cmd("SMEMBERS")
                .arg(format!("{TASKS_KEY}:{run_id}"))
                .query_async::<_, Vec<String>>(&mut conn)
                .await
                .unwrap()
                .iter()
                .map(|t| serde_json::from_str(t).unwrap())
                .collect()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_default_tasks(&self) -> Vec<Task> {
        self.nodes.clone().unwrap()
    }

    #[timed(duration(printer = "debug!"))]
    fn get_default_edges(&self) -> HashSet<(usize, usize)> {
        self.edges.clone().unwrap()
    }

    #[timed(duration(printer = "debug!"))]
    fn get_task_by_id(&self, run_id: usize, task_id: usize) -> Task {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            serde_json::from_str(
                &cmd("GET")
                    .arg(format!("{TASK_KEY}:{run_id}:{task_id}"))
                    .query_async::<_, String>(&mut conn)
                    .await
                    .unwrap(),
            )
            .unwrap()
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
    ) -> usize {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            let task_id = cmd("INCR")
                .arg(format!("{TASK_ID_KEY}:{run_id}"))
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap()
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
            };
            cmd("SADD")
                .arg(format!("{TASKS_KEY}:{run_id}"))
                .arg(serde_json::to_string(&task).unwrap())
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap();
            cmd("SET")
                .arg(format!("{TASK_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&task).unwrap())
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
            cmd("SET")
                .arg(format!("{TEMPLATE_ARGS_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&task.template_args).unwrap())
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
            self.set_task_status(run_id, task_id, TaskStatus::Pending);
            task_id
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn get_template_args(&self, run_id: usize, task_id: usize) -> serde_json::Value {
        let task = self.get_task_by_id(run_id, task_id);
        task.template_args
    }

    #[timed(duration(printer = "debug!"))]
    fn set_template_args(&mut self, run_id: usize, task_id: usize, template_args_str: &str) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();
            let mut task = self.get_task_by_id(run_id, task_id);
            task.template_args = serde_json::from_str(template_args_str).unwrap();

            cmd("SET")
                .arg(format!("{TASK_KEY}:{run_id}:{task_id}"))
                .arg(serde_json::to_string(&task).unwrap())
                .query_async::<_, String>(&mut conn)
                .await
                .unwrap();
        });
    }

    #[timed(duration(printer = "debug!"))]
    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask> {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            let parallel_task_count = cmd("SCARD")
                .arg("tmpqueue") // TODO timeout arg
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap();

            let max_threads = 10;

            if parallel_task_count >= max_threads {
                return None;
            }

            let res = cmd("ZPOPMIN")
                .arg(&["queue".to_string(), "1".to_string()]) // TODO timeout arg
                .query_async::<_, Vec<String>>(&mut conn)
                .await;

            if let Ok(vec) = &res {
                if !vec.is_empty() {
                    cmd("SADD")
                        .arg(&["tmpqueue".to_string(), vec[0].to_string()])
                        .query_async::<_, ()>(&mut conn)
                        .await
                        .unwrap();
                    return Some(OrderedQueuedTask {
                        score: vec[1].parse().unwrap(),
                        queued_task: serde_json::from_str(&vec[0]).unwrap(),
                    });
                }
            } else {
                println!("{:#?}", res.unwrap_err().detail());
            }

            None
        })
    }

    // #[timed(duration(printer = "debug!"))]
    // fn push_priority_queue(&mut self, queued_task: OrderedQueuedTask) {
    //     tokio::task::block_in_place(|| {
    //         tokio::runtime::Handle::current().block_on(async {
    //             let mut conn = self.pool.get().await.unwrap();
    //             cmd("ZADD")
    //                 .arg("queue")
    //                 .arg(queued_task.score)
    //                 .arg(serde_json::to_string(&queued_task.queued_task).unwrap())
    //                 .query_async::<_, f64>(&mut conn)
    //                 .await
    //                 .unwrap();
    //         });
    //     });
    // }

    fn get_task_depth(&mut self, run_id: usize, task_id: usize) -> usize {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            if !cmd("EXISTS")
                .arg(format!("{DEPTH_KEY}:{run_id}:{task_id}"))
                .query_async::<_, bool>(&mut conn)
                .await
                .unwrap()
            {
                let depth = self
                    .get_upstream(run_id, task_id)
                    .iter()
                    .map(|up| self.get_task_depth(run_id, *up) + 1)
                    .max()
                    .unwrap_or(0);
                self.set_task_depth(run_id, task_id, depth);
            }
            cmd("GET")
                .arg(format!("{DEPTH_KEY}:{run_id}:{task_id}"))
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap()
        })
    }

    #[timed(duration(printer = "debug!"))]
    fn set_task_depth(&mut self, run_id: usize, task_id: usize, depth: usize) {
        block_on!({
            let mut conn = self.pool.get().await.unwrap();

            cmd("SET")
                .arg(&[format!("{DEPTH_KEY}:{run_id}:{task_id}"), depth.to_string()])
                .query_async::<_, ()>(&mut conn)
                .await
                .unwrap();
        });
    }

    #[timed(duration(printer = "debug!"))]
    fn enqueue_task(
        &mut self,
        run_id: usize,
        task_id: usize,
        scheduled_date_for_dag_run: DateTime<Utc>,
        dag_name: String,
    ) {
        let attempt: usize = self.get_attempt_by_task_id(run_id, task_id);

        block_on!({
            let depth = self.get_task_depth(run_id, task_id);
            let mut conn = self.pool.get().await.unwrap();

            cmd("ZADD")
                .arg(&[
                    "queue".to_string(),
                    depth.to_string(),
                    serde_json::to_string(&QueuedTask {
                        task_id,
                        run_id,
                        dag_name,
                        scheduled_date_for_dag_run,
                        attempt,
                    })
                    .unwrap(),
                ])
                .query_async::<_, usize>(&mut conn)
                .await
                .unwrap();
        });
    }

    #[timed(duration(printer = "debug!"))]
    fn print_priority_queue(&mut self) {}

    fn take_last_stdout_line(
        &mut self,
        run_id: usize,
        task_id: usize,
        attempt: usize,
    ) -> Box<dyn Fn() -> String + Send> {
        let pool = self.pool.clone();
        Box::new(move || {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut conn = pool.get().await.unwrap();
                    cmd("RPOP")
                        .arg(format!("{LOG_KEY}:{run_id}:{task_id}:{attempt}"))
                        .arg(1)
                        .query_async::<_, Vec<String>>(&mut conn)
                        .await
                        .unwrap_or_default()
                        .first()
                        .unwrap_or(&"null".into())
                        .to_string()
                })
            })
        })
    }
}

#[derive(Clone)]
pub struct LocalRunner<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> {
    pub backend: Box<U>,
    pub tpt_path: String,
    pub executor_path: String,
    pub max_parallelism: usize,
}
