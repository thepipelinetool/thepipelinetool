use std::{env, path::Path, time::Duration};
use thepipelinetool_runner::{blanket::BlanketRunner, Runner};
use thepipelinetool_server::{
    _get_dag_path_by_name, get_redis_pool, redis_runner::RedisRunner, tpt_installed,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    assert!(tpt_installed());

    let pool = get_redis_pool();
    let mut dummy = RedisRunner::dummy(pool.clone());
    let tpt_path = Path::new("tpt").to_path_buf();

    loop {
        if let Some(ordered_queued_task) = dummy.pop_priority_queue() {
            env::set_var("run_id", ordered_queued_task.queued_task.run_id.to_string());

            let mut runner = RedisRunner::from_local_dag(
                &ordered_queued_task.queued_task.dag_name,
                pool.clone(),
            );

            runner.work(
                ordered_queued_task.queued_task.run_id,
                &ordered_queued_task,
                _get_dag_path_by_name(&ordered_queued_task.queued_task.dag_name),
                tpt_path.clone(),
            );
            runner.remove_from_temp_queue(&ordered_queued_task.queued_task);
        } else {
            sleep(Duration::new(2, 0)).await;
        }
    }
}
