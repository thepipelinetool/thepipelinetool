use deadpool_redis::Pool;
use futures::executor;
use std::{
    env,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};
use thepipelinetool_core::dev::OrderedQueuedTask;
use thepipelinetool_runner::{blanket::BlanketRunner, spawn_executor, Executor, Runner};
use thepipelinetool_server::{
    _get_dag_path_by_name, get_redis_pool,
    redis_runner::RedisRunner,
    statics::{_get_default_edges, _get_default_tasks},
    tpt_installed,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    assert!(tpt_installed());

    let pool = get_redis_pool();
    let mut dummy = RedisRunner::dummy(pool.clone());

    loop {
        let num_threads = 10usize;
        let mut thread_count = dummy.get_running_tasks_count().await;

        if dummy.get_queue_length() == 0 || thread_count >= num_threads {
            sleep(Duration::new(2, 0)).await;
            continue;
        }

        let (tx, rx) = channel();

        'inner: for _ in 0..num_threads {
            if let Some(ordered_queued_task) = dummy.pop_priority_queue() {
                spawn_executor(tx.clone(), move || execute(ordered_queued_task));

                thread_count += 1;
                if thread_count >= num_threads {
                    break 'inner;
                }
            } else {
                break 'inner;
            }
        }

        'inner: for _ in rx.iter() {
            thread_count -= 1;

            if let Some(ordered_queued_task) = dummy.pop_priority_queue() {
                spawn_executor(tx.clone(), move || execute(ordered_queued_task));

                thread_count += 1;

                if thread_count >= num_threads {
                    continue 'inner;
                }
            }
            if thread_count == 0 {
                drop(tx);
                break 'inner;
            }
        }
    }
}

fn execute(ordered_queued_task: OrderedQueuedTask) {
    let executor = Executor::Local;
    let tpt_path = "tpt".to_string();
    let dag_path = _get_dag_path_by_name(&ordered_queued_task.queued_task.dag_name).unwrap();

    match executor {
        Executor::Local => {
            let pool = get_redis_pool();

            let nodes = _get_default_tasks(&ordered_queued_task.queued_task.dag_name).unwrap();
            let edges = _get_default_edges(&ordered_queued_task.queued_task.dag_name).unwrap();

            RedisRunner::from(
                &ordered_queued_task.queued_task.dag_name,
                nodes,
                edges,
                pool.clone(),
            )
            .work(
                ordered_queued_task.queued_task.run_id,
                &ordered_queued_task,
                dag_path,
                tpt_path,
                ordered_queued_task.queued_task.scheduled_date_for_dag_run,
            );
        }
        Executor::Docker => {
            todo!()
        }
        Executor::Kubernetes => {
            todo!()
        }
    }
}
