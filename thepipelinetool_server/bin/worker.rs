use futures::executor;
use std::{
    env,
    thread::{self, JoinHandle},
    time::Duration,
};
use thepipelinetool_runner::backend::Backend;
use thepipelinetool_runner::run;
use thepipelinetool_server::{
    get_redis_pool,
    redis::{RedisBackend, RedisRunner},
    tpt_installed,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    assert!(tpt_installed());

    // TODO read from env
    let max_parallelism = 10usize;

    let mut runner = RedisRunner {
        backend: Box::new(RedisBackend::dummy(get_redis_pool())),
        tpt_path: env::args().next().unwrap(),
        executor_path: "tpt_executor".to_string(),
        max_parallelism,
    };
    loop {
        if runner.backend.get_queue_length() == 0
            || runner.backend.get_running_tasks_count().await >= max_parallelism
        {
            sleep(Duration::new(2, 0)).await;
            continue;
        }

        run(&mut runner);
    }
}

// fn _spawn_thread(mut f: Box<dyn FnMut() + Send + 'static>) -> JoinHandle<()> {
//     // tokio::task::block_in_place(|| {
//     //     tokio::runtime::Handle::current().block_on(async {
//     //         f();
//     //     });
//     // });
//     // let handle = tokio::spawn(async move {
//     //     f();
//     // });
//     // thread::spawn(move || {
//         let handle = tokio::runtime::Handle::current();
//         let _ = handle.enter();
//         executor::block_on(async {
//             f();
//         });
//     // })
//     // handle.await;
//     thread::spawn(|| {
       
//     })
// }

#[derive(Clone)]
enum Executor {
    Local,
    // Docker,
    // Kubernetes,
}

// fn execute(ordered_queued_task: OrderedQueuedTask) {
//     let executor = Executor::Local;
//     let tpt_path = "tpt".to_string();
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
