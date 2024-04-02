use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};

use deadpool_redis::Pool;
use saffron::Cron;
use tokio::time::sleep;

use crate::{_get_dags, _trigger_run_from_schedules, statics::_get_options};

pub fn spawn_scheduler(server_start_date: DateTime<Utc>, pool: Pool) {
    tokio::spawn(async move {
        let mut last_checked_name: HashMap<String, DateTime<Utc>> = HashMap::new();
        let pool = pool.clone();

        loop {
            'inner: for dag_name in _get_dags() {
                let options = _get_options(&dag_name).unwrap();

                let last_checked = **last_checked_name
                    .get(&dag_name)
                    .get_or_insert(&server_start_date);

                last_checked_name.insert(dag_name.clone(), Utc::now());

                if options.schedule.is_none() {
                    continue 'inner;
                }
                let cron = &options.schedule.unwrap().parse::<Cron>().unwrap();
                if !cron.any() {
                    println!("Cron will never match any given time!");
                    continue 'inner;
                }
                // println!("checking for schedules: {dag_name} {up_to}");

                if let Some(end_date) = options.end_date {
                    if end_date <= last_checked {
                        continue 'inner;
                    }
                }

                if let Some(start_date) = options.start_date {
                    if start_date >= last_checked {
                        continue 'inner;
                    }
                }

                _trigger_run_from_schedules(
                    &dag_name,
                    server_start_date,
                    cron,
                    cron.clone().iter_from(last_checked),
                    options.end_date,
                    pool.clone(),
                )
                .await;
            }

            // TODO read from env
            sleep(Duration::new(5, 0)).await;
        }
    });
}
