use std::{process::Command, sync::mpsc::channel, thread, time::Duration};
// use thepipelinetool_runner::run;
use anyhow::Result;
use thepipelinetool_runner::{
    backend::Backend, blanket_backend::BlanketBackend, get_tpt_executor_command,
};
use thepipelinetool_server::{
    env::{get_executor_type, get_max_parallelism, get_redis_url},
    get_redis_pool,
    redis_backend::RedisBackend,
    Executor,
};
use thepipelinetool_utils::spawn;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    let max_parallelism = get_max_parallelism()?;
    let executor = get_executor_type()?;
    let backend = RedisBackend::dummy(get_redis_pool()?);

    println!("Running tpt worker with '{:?}' executor type", executor);
    println!("Connected to redis at {}", get_redis_url());

    loop {
        let mut backend = backend.clone();

        // TODO read from env
        sleep(Duration::from_nanos(1)).await;

        tokio::spawn(async move { run_in_memory(max_parallelism, executor, &mut backend).await });
    }
}

// async fn _work(
//     max_parallelism: usize,
//     executor: Executor,
//     backend: &mut RedisBackend,
// ) -> Result<()> {
//     run_in_memory(backend, max_parallelism, tpt_path);
//     // for _ in backend.get_running_tasks_count().await?..max_parallelism {
//     //     let temp_queued_task = backend.pop_priority_queue()?;
//     //     if temp_queued_task.is_none() {
//     //         return Ok(());
//     //     }

//     //     let temp_queued_task = temp_queued_task.expect("");
//     //     thread::spawn(move || match executor {
//     //         Executor::Local => {
//     //             let mut cmd = Command::new(get_tpt_executor_command());
//     //             cmd.arg(serde_json::to_string(&temp_queued_task).unwrap());
//     //             let _ = spawn(
//     //                 cmd,
//     //                 None,
//     //                 Box::new(|x| {
//     //                     print!("{x}");
//     //                     Ok(())
//     //                 }),
//     //                 Box::new(|x| {
//     //                     eprint!("{x}");
//     //                     Ok(())
//     //                 }),
//     //             );
//     //         }
//     //         Executor::Docker => {
//     //             let mut cmd = Command::new("docker");
//     //             cmd.args(&["run", "-e"]);
//     //             cmd.arg(format!("REDIS_URL={}", get_redis_url()));
//     //             cmd.arg("--network=thepipelinetool_default");
//     //             cmd.arg("executor");
//     //             cmd.arg(serde_json::to_string(&temp_queued_task).unwrap());

//     //             let _ = spawn(
//     //                 cmd,
//     //                 None,
//     //                 Box::new(|x| {
//     //                     print!("{x}");
//     //                     Ok(())
//     //                 }),
//     //                 Box::new(|x| {
//     //                     eprint!("{x}");
//     //                     Ok(())
//     //                 }),
//     //             );
//     //         }
//     //         Executor::Kubernetes => todo!(),
//     //     });
//     // }
//     Ok(())
// }

pub async fn run_in_memory(
    max_parallelism: usize,
    executor: Executor,
    backend: &mut RedisBackend,
) -> Result<()> {
    let (tx, rx) = channel();
    let mut current_parallel_tasks_count = 0;

    for _ in backend.get_running_tasks_count().await?..max_parallelism {

        if let Some(temp_queued_task) = backend.pop_priority_queue().unwrap() {
            let tx = tx.clone();
            // let mut backend = backend.clone();
            // let tpt_path = tpt_path.clone();

            thread::spawn(move || {
                // backend.work(&temp_queued_task, tpt_path).unwrap();
                // backend.remove_from_temp_queue(&temp_queued_task).unwrap();

                match executor {
                    Executor::Local => {
                        let mut cmd = Command::new(get_tpt_executor_command());
                        cmd.arg(serde_json::to_string(&temp_queued_task).unwrap());
                        let _ = spawn(
                            cmd,
                            None,
                            Box::new(|x| {
                                print!("{x}");
                                Ok(())
                            }),
                            Box::new(|x| {
                                eprint!("{x}");
                                Ok(())
                            }),
                        );
                    }
                    Executor::Docker => {
                        let mut cmd = Command::new("docker");
                        cmd.args(&["run", "-e"]);
                        cmd.arg(format!("REDIS_URL={}", get_redis_url()));
                        cmd.arg("--network=thepipelinetool_default");
                        cmd.arg("executor");
                        cmd.arg(serde_json::to_string(&temp_queued_task).unwrap());

                        let _ = spawn(
                            cmd,
                            None,
                            Box::new(|x| {
                                print!("{x}");
                                Ok(())
                            }),
                            Box::new(|x| {
                                eprint!("{x}");
                                Ok(())
                            }),
                        );
                    }
                    Executor::Kubernetes => todo!(),
                }
                tx.send(()).unwrap();
            });

            current_parallel_tasks_count += 1;
            if current_parallel_tasks_count >= max_parallelism {
                break;
            }
        } else {
            break;
        }
    }

    if current_parallel_tasks_count == 0 {
        drop(tx);
        return Ok(());
    }

    for _ in rx.iter() {
        current_parallel_tasks_count -= 1;

        if let Some(temp_queued_task) = backend.pop_priority_queue().unwrap() {
            let tx = tx.clone();
            let mut backend = backend.clone();
            // let tpt_path = tpt_path.clone();

            thread::spawn(move || {

                match executor {
                    Executor::Local => {
                        let mut cmd = Command::new(get_tpt_executor_command());
                        cmd.arg(serde_json::to_string(&temp_queued_task).unwrap());
                        let _ = spawn(
                            cmd,
                            None,
                            Box::new(|x| {
                                print!("{x}");
                                Ok(())
                            }),
                            Box::new(|x| {
                                eprint!("{x}");
                                Ok(())
                            }),
                        );
                    }
                    Executor::Docker => {
                        let mut cmd = Command::new("docker");
                        cmd.args(&["run", "-e"]);
                        cmd.arg(format!("REDIS_URL={}", get_redis_url()));
                        cmd.arg("--network=thepipelinetool_default");
                        cmd.arg("executor");
                        cmd.arg(serde_json::to_string(&temp_queued_task).unwrap());

                        let _ = spawn(
                            cmd,
                            None,
                            Box::new(|x| {
                                print!("{x}");
                                Ok(())
                            }),
                            Box::new(|x| {
                                eprint!("{x}");
                                Ok(())
                            }),
                        );
                    }
                    Executor::Kubernetes => todo!(),
                }
                tx.send(()).unwrap();
            });
            current_parallel_tasks_count += 1;

            if current_parallel_tasks_count >= max_parallelism {
                continue;
            }
        }
        if current_parallel_tasks_count == 0 {
            drop(tx);
            break;
        }
    }
    Ok(())
}
