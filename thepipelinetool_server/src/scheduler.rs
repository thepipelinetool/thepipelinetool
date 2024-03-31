use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};

use deadpool_redis::Pool;
use saffron::Cron;
use tokio::time::sleep;

use crate::{_get_dags, _get_hash, _trigger_run, redis_runner::RedisRunner, statics::_get_options};

pub fn spawn_scheduler(up_to: &DateTime<Utc>, pool: Pool) {
    let up_to_initial = *up_to;

    tokio::spawn(async move {
        let mut last_checked: HashMap<String, DateTime<Utc>> = HashMap::new();
        loop {
            let dags = _get_dags();

            for dag_name in dags {
                let pool = pool.clone();

                let options = _get_options(&dag_name);
                let up_to = **last_checked.get(&dag_name).get_or_insert(&up_to_initial);

                last_checked.insert(dag_name.clone(), Utc::now());

                if let Some(schedule) = &options.schedule {
                    match schedule.parse::<Cron>() {
                        Ok(cron) => {
                            if !cron.any() {
                                println!("Cron will never match any given time!");
                                return;
                            }
                            // println!("checking for schedules: {dag_name} {up_to}");

                            if let Some(end_date) = options.end_date {
                                if end_date <= up_to {
                                    return;
                                }
                            }

                            if let Some(start_date) = options.start_date {
                                if start_date >= up_to {
                                    return;
                                }
                            }

                            let futures = cron.clone().iter_from(up_to);

                            'inner: for time in futures {
                                if !cron.contains(time) {
                                    println!("Failed check! Cron does not contain {}.", time);
                                    break 'inner;
                                }
                                if time >= Utc::now() {
                                    break 'inner;
                                }
                                if let Some(end_date) = options.end_date {
                                    if time > end_date {
                                        break 'inner;
                                    }
                                }
                                // check if date is already in db
                                if RedisRunner::contains_logical_date(
                                    &dag_name,
                                    &_get_hash(&dag_name),
                                    time,
                                    pool.clone(),
                                )
                                .await
                                {
                                    continue 'inner;
                                }

                                _trigger_run(&dag_name, time, pool.clone()).await;
                                println!("scheduling {} {dag_name}", time.format("%F %R"));
                            }
                        }
                        Err(err) => println!("{err}: {schedule}"),
                    }
                }
            }

            // TODO read from env
            sleep(Duration::new(5, 0)).await;
        }
    });
}
