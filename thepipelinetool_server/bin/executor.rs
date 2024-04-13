use std::env;

use anyhow::Result;
use thepipelinetool_core::dev::OrderedQueuedTask;
use thepipelinetool_runner::backend::Backend;
use thepipelinetool_runner::{blanket_backend::BlanketBackend, get_pipeline_path_by_name};
use thepipelinetool_server::{
    env::get_tpt_command,
    get_redis_pool,
    redis_backend::RedisBackend,
    statics::{_get_default_edges, _get_default_tasks},
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let ordered_queued_task: OrderedQueuedTask = serde_json::from_str(&args[1])?;

    let tasks = _get_default_tasks(&ordered_queued_task.queued_task.pipeline_name)?;
    let edges = _get_default_edges(&ordered_queued_task.queued_task.pipeline_name)?;

    let mut backend = RedisBackend::from(tasks, edges, get_redis_pool()?);
    backend.work(
        ordered_queued_task.queued_task.run_id,
        &ordered_queued_task,
        get_pipeline_path_by_name(&ordered_queued_task.queued_task.pipeline_name)?,
        get_tpt_command(),
        ordered_queued_task.queued_task.scheduled_date_for_run,
    )?;
    backend.remove_from_temp_queue(&ordered_queued_task.queued_task)?;
    Ok(())
}
