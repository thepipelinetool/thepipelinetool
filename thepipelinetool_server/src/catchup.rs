use chrono::{DateTime, Utc};
use deadpool_redis::Pool;
use saffron::Cron;
use thepipelinetool_runner::options::DagOptions;

use crate::{
    _get_dags, _get_schedules_for_catchup, _trigger_run_from_schedules, statics::_get_options,
};

pub fn spawn_catchup(server_start_date: DateTime<Utc>, pool: Pool) {
    tokio::spawn(async move {
        let pool = pool.clone();

        for dag_name in _get_dags() {
            // tokio::spawn(async move {
            let options: DagOptions = _get_options(&dag_name).unwrap();

            if options.schedule.is_none() {
                continue;
            }

            let cron = &options.schedule.clone().unwrap().parse::<Cron>().unwrap();

            if !cron.any() {
                println!("Cron will never match any given time!");
                continue;
            }
            println!("checking for catchup: {dag_name}");

            if let Some(start_date) = options.get_start_date_with_timezone() {
                if start_date >= server_start_date {
                    continue;
                }
            }
            _trigger_run_from_schedules(
                &dag_name,
                server_start_date,
                cron,
                _get_schedules_for_catchup(
                    cron,
                    options.get_start_date_with_timezone(),
                    options.should_catchup,
                    server_start_date,
                ),
                options.get_end_date_with_timezone(),
                pool.clone(),
            )
            .await;
        }
    });
}
