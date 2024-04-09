use std::{
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
// use thepipelinetool_runner::run;
use thepipelinetool_runner::{
    backend::Backend, blanket_backend::BlanketBackend, get_dag_path_by_name,
    get_tpt_executor_command, Executor,
};
use thepipelinetool_server::{
    env::{get_max_parallelism, get_tpt_command},
    get_redis_pool,
    redis_backend::RedisBackend,
};
use thepipelinetool_utils::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    let max_parallelism = get_max_parallelism();
    let mut backend = RedisBackend::dummy(get_redis_pool());

    run(
        &mut backend,
        max_parallelism,
        None,
        Some(get_tpt_command()),
        Executor::Local,
    )
    .await;
    // sleep(Duration::new(2, 0)).await;
    // dbg!(backend.get_running_tasks_count().await);
}

pub async fn run(
    backend: &mut RedisBackend,
    max_parallelism: usize,
    dag_path: Option<String>,
    tpt_path: Option<String>,
    executor: Executor,
) {
    // let (tx, rx) = channel();
    let immediate = Arc::new(AtomicBool::new(true));

    // let par = Arc::new(AtomicUsize::new(0));

    loop {
        // let immediate = immediate.clone();

        let mut backend = backend.clone();
        // let (dag_path, tpt_path) = (dag_path.clone(), tpt_path.clone());

        // let par = par.clone();

        // if immediate.load(Ordering::SeqCst) == false {
        // && par.load(Ordering::SeqCst) >= max_parallelism {
        sleep(Duration::from_millis(250)).await;
        // continue;
        // }
        // immediate.store(false, Ordering::SeqCst);

        tokio::spawn(async move {
            // let immediate = immediate.clone();

            for _ in dbg!(backend.get_running_tasks_count().await)..max_parallelism {
            // par.fetch_add(1, Ordering::SeqCst);
            let ordered_queued_task = backend.pop_priority_queue();
            if ordered_queued_task.is_none() {
                // sleep(Duration::new(2, 0)).await;
                // par.fetch_sub(1, Ordering::SeqCst);
                return;
            }
            // let tx = tx.clone();

            let ordered_queued_task = ordered_queued_task.unwrap();

            // let mut backend = backend.clone();
            // let (dag_path, tpt_path) = (dag_path.clone(), tpt_path.clone());
            // let dag_path = dag_path.unwrap_or(
            //     get_dag_path_by_name(&ordered_queued_task.queued_task.dag_name)
            //         .unwrap()
            //         .to_str()
            //         .unwrap()
            //         .to_string(),
            // );

            let mut cmd = Command::new(get_tpt_executor_command());
            cmd.arg(serde_json::to_string(&ordered_queued_task).unwrap());
            let _ = spawn(
                cmd,
                Box::new(|x| print!("{x}")),
                Box::new(|x| eprint!("{x}")),
            );
            // immediate.store(true, Ordering::SeqCst);
            // par.fetch_sub(1, Ordering::SeqCst);
            }
        });

        // current_parallel_tasks_count += 1;
        // if current_parallel_tasks_count >= max_parallelism {
        //     break;
        // }
    }
}

// fn execute(ordered_queued_task: OrderedQueuedTask) {
//     let executor = Executor::Local;
//     let dag_path = _get_dag_path_by_name(&ordered_queued_task.queued_task.dag_name).unwrap();

//     match executor {
//         Executor::Local => {
//             let nodes = _get_default_tasks(&ordered_queued_task.queued_task.dag_name).unwrap();
//             let edges = _get_default_edges(&ordered_queued_task.queued_task.dag_name).unwrap();

//             RedisBackend::from(
//                 &ordered_queued_task.queued_task.dag_name,
//                 // nodes,
//                 // edges,
//                 // get_redis_pool(),
//             )
//             .work(
//                 ordered_queued_task.queued_task.run_id,
//                 &ordered_queued_task,
//                 dag_path,
//                 tpt_path,
//                 ordered_queued_task.queued_task.scheduled_date_for_dag_run,
//             );
//         } // Executor::Docker => {
//           //     todo!()
//           // }
//           // Executor::Kubernetes => {
//           //     todo!()
//           // }
//     }
// }
