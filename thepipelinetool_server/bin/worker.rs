use std::{process::Command, time::Duration};
// use thepipelinetool_runner::run;
use anyhow::Result;
use thepipelinetool_runner::{
    backend::Backend, get_tpt_executor_command,
};
use thepipelinetool_server::{
    env::{
        get_executor_image, get_executor_type, get_max_parallelism, get_redis_url,
        get_worker_loop_interval,
    },
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
    let loop_interval = Duration::from_millis(get_worker_loop_interval()?);

    println!("Running tpt worker with '{:?}' executor type", executor);
    println!("Connected to redis at {}", get_redis_url());

    loop {
        let mut backend = backend.clone();

        sleep(loop_interval).await;

        tokio::spawn(async move { work(max_parallelism, executor, &mut backend).await });
    }
}

async fn work(
    max_parallelism: usize,
    executor: Executor,
    backend: &mut RedisBackend,
) -> Result<()> {
    if backend.get_running_tasks_count().await? < max_parallelism {
        let temp_queued_task = backend.pop_priority_queue()?;
        if temp_queued_task.is_none() {
            return Ok(());
        }

        let temp_queued_task = temp_queued_task.expect("");
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
                cmd.arg(get_executor_image()?);
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
    }
    Ok(())
}
