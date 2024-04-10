use chrono::{DateTime, Utc};
use deadpool_redis::Pool;
use saffron::Cron;
use thepipelinetool_runner::options::DagOptions;

use crate::{_get_dags, _trigger_run_from_schedules, statics::_get_options};
use anyhow::anyhow;
use anyhow::Result;

async fn _spawn_catchup(server_start_date: DateTime<Utc>, pool: Pool) -> Result<()> {
    let pool = pool.clone();

    for dag_name in _get_dags()? {
        // tokio::spawn(async move {
        let options: DagOptions = _get_options(&dag_name)?;

        if options.schedule.is_none() {
            continue;
        }

        let cron = &options.schedule.clone().expect("").parse::<Cron>();
        if cron.is_err() {
            // TODO check for correct cron on read
            continue;
        }
        let cron = cron.as_ref().expect("");

        if !cron.any() {
            println!("Cron will never match any given time!");
            continue;
        }
        if !options.should_catchup {
            continue;
        }
        if let Some(start_date) = options.get_start_date_with_timezone() {
            if start_date >= server_start_date {
                continue;
            }
        } else {
            continue;
        }
        println!("checking for catchup: {dag_name}");
        _trigger_run_from_schedules(
            &dag_name,
            server_start_date,
            cron,
            cron.clone().iter_from(
                options
                    .get_start_date_with_timezone()
                    .ok_or(anyhow!(format!("{dag_name} does not exist")))?,
            ),
            options.get_end_date_with_timezone(),
            pool.clone(),
        )
        .await?;
    }
    Ok(())
}

pub fn spawn_catchup(server_start_date: DateTime<Utc>, pool: Pool) {
    tokio::spawn(async move { _spawn_catchup(server_start_date, pool).await });
}
