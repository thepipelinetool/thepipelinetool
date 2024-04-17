use std::collections::HashSet;

use deadpool::Runtime;
use deadpool_redis::{Config, Pool};
use env::get_redis_url;
use redis_backend::RedisBackend;
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::run::Run;
use thepipelinetool_runner::{backend::Backend, blanket_backend::BlanketBackend};

use anyhow::Result;

pub mod check_timeout;
pub mod env;
pub mod redis_backend;
pub mod routes;
pub mod scheduler;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Executor {
    Local,
    Docker,
    Kubernetes,
}

pub fn _get_all_tasks_by_run_id(run_id: usize, pool: Pool) -> Result<Vec<Task>> {
    RedisBackend::dummy(pool).get_all_tasks(run_id)
}

pub fn _get_task_by_id(run_id: usize, task_id: usize, pool: Pool) -> Result<Task> {
    RedisBackend::dummy(pool).get_task_by_id(run_id, task_id)
}

pub async fn _get_all_task_results(
    run_id: usize,
    task_id: usize,
    pool: Pool,
) -> Result<Vec<TaskResult>> {
    RedisBackend::get_all_results(run_id, task_id, pool).await
}

pub fn _get_task_status(run_id: usize, task_id: usize, pool: Pool) -> Result<TaskStatus> {
    RedisBackend::dummy(pool).get_task_status(run_id, task_id)
}

pub fn _get_run_status(run_id: usize, pool: Pool) -> Result<i32> {
    RedisBackend::dummy(pool).get_run_status(run_id)
}

pub fn _get_task_result(run_id: usize, task_id: usize, pool: Pool) -> Result<TaskResult> {
    RedisBackend::dummy(pool).get_task_result(run_id, task_id)
}

pub fn get_redis_pool() -> Result<Pool> {
    let cfg = Config::from_url(get_redis_url());
    Ok(cfg.create_pool(Some(Runtime::Tokio1))?)
}

pub async fn _get_next_run(pipeline_name: &str, pool: Pool) -> Result<Option<String>> {
    RedisBackend::get_next_run(pipeline_name, pool).await
}

pub async fn _get_last_run(pipeline_name: &str, pool: Pool) -> Result<Vec<Run>> {
    let r = RedisBackend::get_last_run(pipeline_name, pool).await?;

    Ok(match r {
        Some(run) => vec![run],
        None => vec![],
    })
}

pub async fn _get_recent_runs(pipeline_name: &str, pool: Pool) -> Result<Vec<Run>> {
    RedisBackend::get_recent_runs(pipeline_name, pool).await
}

pub async fn _get_pipelines(pool: Pool) -> Result<HashSet<String>> {
    RedisBackend::get_pipelines(pool).await
}
