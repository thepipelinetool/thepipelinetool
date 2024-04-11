use std::{
    collections::HashSet,
    time::Duration,
};

use chrono::{DateTime, Utc};

use deadpool_redis::Pool;
use saffron::{Cron, CronTimesIter};
use thepipelinetool_runner::{backend::Backend, blanket_backend::BlanketBackend};
use tokio::time::sleep;

use anyhow::Result;

use crate::{
    _get_pipelines,
    redis_backend::RedisBackend,
    statics::{_get_default_edges, _get_default_tasks, _get_hash, _get_options},
};

pub async fn scheduler(pool: Pool) -> Result<()> {
    let pool = pool.clone();
    let mut spawned_schedulers = HashSet::new();

    loop {
        'inner: for pipeline_name in _get_pipelines()? {
            if spawned_schedulers.contains(&pipeline_name) {
                // scheduler for this pipeline already spawned
                continue;
            }
            spawned_schedulers.insert(pipeline_name.clone());
            let options = _get_options(&pipeline_name)?;

            if options.schedule.is_none() {
                // no scheduling for this pipeline
                continue 'inner;
            }
            let cron = if let Ok(cron) = (&options.schedule).clone().expect("").parse::<Cron>() {
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
            tokio::spawn(async move {
                _scheduler(
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
                .await
            });
        }

        // TODO read from env
        sleep(Duration::new(5, 0)).await;
    }
}

pub async fn _scheduler(
    pipeline_name: &str,
    // server_start_date: DateTime<Utc>,
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
            let delay = (scheduled_date - now).to_std()?;
            tokio::time::sleep(delay).await;
        }

        // check if date is already in db
        if RedisBackend::contains_logical_date(
            pipeline_name,
            &_get_hash(pipeline_name)?,
            scheduled_date,
            pool.clone(),
        )
        .await?
        {
            continue;
        }
        let nodes = _get_default_tasks(pipeline_name)?;
        let edges = _get_default_edges(pipeline_name)?;
        let hash = _get_hash(pipeline_name)?;

        let mut backend = RedisBackend::from(nodes, edges, pool.clone());
        let run_id = backend.create_new_run(pipeline_name, &hash, scheduled_date)?;
        backend.enqueue_run(run_id, pipeline_name, &hash, scheduled_date, None)?;
        println!(
            "scheduling catchup {pipeline_name} {}",
            scheduled_date.format("%F %R")
        );
    }

    // TODO set next run to none
    Ok(())
}
