use std::{env, time::Duration};
use thepipelinetool_runner::run;
use thepipelinetool_runner::{backend::Backend, get_max_parallelism};
use thepipelinetool_server::{
    get_executor_command, get_redis_pool,
    redis::{LocalRunner, RedisBackend},
};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    let max_parallelism = get_max_parallelism();

    let mut runner = LocalRunner {
        backend: Box::new(RedisBackend::dummy(get_redis_pool())),
        tpt_path: env::args().next().unwrap(),
        executor_path: get_executor_command(),
    };
    loop {
        if runner.backend.get_queue_length() == 0
            || runner.backend.get_running_tasks_count().await >= max_parallelism
        {
            sleep(Duration::new(2, 0)).await;
            continue;
        }

        run(&mut runner, max_parallelism);
    }
}

#[derive(Clone)]
enum Executor {
    Local,
    // Docker,
    // Kubernetes,
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
