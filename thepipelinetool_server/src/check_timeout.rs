use std::time::Duration;

use chrono::Utc;

use deadpool_redis::Pool;
use thepipelinetool_core::dev::TaskResult;
use thepipelinetool_runner::{backend::Backend, blanket_backend::BlanketBackend};
use tokio::time::sleep;

use crate::redis::RedisBackend;

pub fn spawn_check_timeout(pool: Pool) {
    tokio::spawn(async move {
        let mut dummy = RedisBackend::dummy(pool);
        loop {
            for queued_task in dummy.get_temp_queue().await {
                let task = dummy.get_task_by_id(queued_task.run_id, queued_task.task_id);
                if let Some(timeout) = task.options.timeout {
                    let now = Utc::now();
                    if (now - queued_task.scheduled_date_for_dag_run)
                        .to_std()
                        .unwrap()
                        > timeout
                    {
                        let result = TaskResult::premature_error(
                            task.id,
                            queued_task.attempt,
                            task.options.max_attempts,
                            task.name.clone(),
                            task.function.clone(),
                            "timed out".to_string(),
                            task.is_branch,
                            task.options.is_sensor,
                            queued_task.scheduled_date_for_dag_run,
                        );

                        dummy.handle_task_result(queued_task.run_id, result, &queued_task);
                    }
                } else {
                    continue;
                }
            }

            // TODO read from env
            sleep(Duration::new(5, 0)).await;
        }
    });
}
