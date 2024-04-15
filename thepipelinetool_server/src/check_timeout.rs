use std::time::Duration;

use chrono::Utc;

use deadpool_redis::Pool;
use thepipelinetool_core::dev::TaskResult;
use thepipelinetool_runner::{backend::Backend, blanket_backend::BlanketBackend};
use tokio::time::sleep;

use anyhow::Result;

use crate::{env::get_check_timeout_loop_interval, redis_backend::RedisBackend};

pub async fn check_timeout(pool: Pool) -> Result<()> {
    let mut dummy = RedisBackend::dummy(pool.clone());
    let loop_interval = Duration::new(get_check_timeout_loop_interval()?, 0);

    loop {
        for temp_queued_task in dummy.get_temp_queue().await? {
            let task = dummy.get_task_by_id(
                temp_queued_task.queued_task.run_id,
                temp_queued_task.queued_task.task_id,
            )?;
            if let Some(timeout) = task.options.timeout {
                let now = Utc::now();
                if (now - temp_queued_task.popped_date).to_std()? > timeout {
                    dummy.handle_task_result(
                        temp_queued_task.queued_task.run_id,
                        &temp_queued_task.queued_task,
                        TaskResult::premature_error(
                            task.id,
                            temp_queued_task.queued_task.attempt,
                            task.options.max_attempts,
                            task.name.clone(),
                            task.function.clone(),
                            "timed out".to_string(),
                            task.is_branch,
                            task.options.is_sensor,
                            Some(temp_queued_task.popped_date),
                            Some(now),
                        ),
                    )?;
                }
            }
        }

        sleep(loop_interval).await;
    }
}
