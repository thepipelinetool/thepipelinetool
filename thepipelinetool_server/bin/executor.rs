use std::env;

use thepipelinetool_core::dev::OrderedQueuedTask;
use thepipelinetool_runner::{blanket_backend::BlanketBackend, get_dag_path_by_name};
use thepipelinetool_server::{
    env::get_tpt_command,
    get_redis_pool,
    redis_backend::RedisBackend,
    statics::{_get_default_edges, _get_default_tasks},
};

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<String>>();
    let ordered_queued_task: OrderedQueuedTask = serde_json::from_str(&args[1]).unwrap();

    let tasks = _get_default_tasks(&ordered_queued_task.queued_task.dag_name).unwrap();
    let edges = _get_default_edges(&ordered_queued_task.queued_task.dag_name).unwrap();

    RedisBackend::from(tasks, edges, get_redis_pool()).work(
        ordered_queued_task.queued_task.run_id,
        &ordered_queued_task,
        get_dag_path_by_name(&ordered_queued_task.queued_task.dag_name).unwrap(),
        get_tpt_command(),
        ordered_queued_task.queued_task.scheduled_date_for_dag_run,
    );
}
