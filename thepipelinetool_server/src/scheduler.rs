use std::{collections::HashSet, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};

use deadpool_redis::Pool;
use saffron::{Cron, CronTimesIter};
use thepipelinetool_runner::{backend::Backend, blanket_backend::BlanketBackend};
use tokio::{sync::Mutex, time::sleep};

use anyhow::Result;

use crate::{env::get_scheduler_loop_interval, redis_backend::RedisBackend};

pub async fn scheduler(pool: Pool) -> Result<()> {
    let pool = pool.clone();
    let loop_interval = Duration::new(get_scheduler_loop_interval()?, 0);
    let spawned_schedulers = Arc::new(Mutex::new(HashSet::new()));

    loop {
        // TODO should this watch for updated schedules?
        'inner: for pipeline_name in RedisBackend::get_pipelines(pool.clone()).await? {
            if spawned_schedulers.lock().await.contains(&pipeline_name) {
                // scheduler for this pipeline already spawned
                continue;
            }
            spawned_schedulers
                .lock()
                .await
                .insert(pipeline_name.clone());
            let backend = RedisBackend::from(&pipeline_name, pool.clone());
            let options = backend.get_options().await?;

            if options.schedule.is_none() {
                // no scheduling for this pipeline
                continue 'inner;
            }
            let cron = if let Ok(cron) = options.schedule.clone().expect("").parse::<Cron>() {
                cron
            } else {
                // error parsing cron
                continue 'inner;
            };
            if !cron.any() {
                println!("Cron will never match any given time!");
                continue 'inner;
            }

            let pool = pool.clone();

            // TODO
            // let spawned_schedulers = spawned_schedulers.clone();
            tokio::spawn(async move {
                let _ = _scheduler(
                    &pipeline_name,
                    &cron,
                    cron.clone().iter_from(
                        options
                            .get_catchup_date_with_timezone()
                            .unwrap_or(Utc::now()),
                    ),
                    options.get_end_date_with_timezone(),
                    pool.clone(),
                )
                .await;
                // spawned_schedulers.lock().await.remove(&pipeline_name);
            });
        }

        sleep(loop_interval).await;
    }
}

pub async fn _scheduler(
    pipeline_name: &str,
    cron: &Cron,
    scheduled_dates: CronTimesIter,
    end_date: Option<DateTime<Utc>>,
    pool: Pool,
) -> Result<()> {
    for scheduled_date in scheduled_dates {
        if !cron.contains(scheduled_date) {
            // TODO check if we need this?
            println!("Failed check! Cron does not contain {}.", scheduled_date);
            break;
        }
        if let Some(end_date) = end_date {
            if scheduled_date > end_date {
                break;
            }
        }
        let now = Utc::now();
        if scheduled_date > now {
            // set next run date
            RedisBackend::set_next_run(pipeline_name, Some(scheduled_date), pool.clone()).await?;

            let delay = (scheduled_date - now).to_std()?;
            tokio::time::sleep(delay).await;
        }

        // check if date is already in db
        if RedisBackend::contains_scheduled_date(pipeline_name, scheduled_date, pool.clone())
            .await?
        {
            continue;
        }

        let mut backend = RedisBackend::from(pipeline_name, pool.clone());
        let run = backend.create_new_run(scheduled_date)?;
        backend.enqueue_run(&run, None)?;
        println!(
            "scheduling catchup {pipeline_name} {}",
            scheduled_date.format("%F %R")
        );
    }

    // if this part is reached, that means there are no upcoming runs
    // set next run to None
    RedisBackend::set_next_run(pipeline_name, None, pool).await?;

    Ok(())
}
