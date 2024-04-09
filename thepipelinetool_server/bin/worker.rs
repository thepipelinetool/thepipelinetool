use std::time::Duration;
use thepipelinetool_runner::run;
use thepipelinetool_runner::{backend::Backend, Executor};
use thepipelinetool_server::{
    env::{get_max_parallelism, get_tpt_command},
    get_redis_pool,
    redis_backend::RedisBackend,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    let max_parallelism = get_max_parallelism();
    let mut backend = RedisBackend::dummy(get_redis_pool());

    loop {
        run(
            &mut backend,
            max_parallelism,
            None,
            Some(get_tpt_command()),
            Executor::Local,
        );
        sleep(Duration::new(2, 0)).await;
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
