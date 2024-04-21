use std::env;

use anyhow::Result;
use thepipelinetool_core::dev::TempQueuedTask;
use thepipelinetool_runner::backend::Backend;
use thepipelinetool_runner::blanket_backend::BlanketBackend;
use thepipelinetool_server::{env::get_tpt_command, get_redis_pool, redis_backend::RedisBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let temp_queued_task: TempQueuedTask = serde_json::from_str(&args[1])?;

    let mut backend = RedisBackend::from(
        &temp_queued_task.queued_task.pipeline_name,
        get_redis_pool()?,
    );
    backend.work(&temp_queued_task, get_tpt_command())?;
    backend.remove_from_temp_queue(&temp_queued_task)?;
    Ok(())
}
