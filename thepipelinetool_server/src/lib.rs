use std::{env, fs, io::ErrorKind, path::PathBuf, process::Command};

use chrono::{DateTime, FixedOffset, Utc};
use deadpool::Runtime;
use deadpool_redis::{Config, Pool};
use log::{debug, info};
use redis_runner::{RedisRunner, Run};
use saffron::{Cron, CronTimesIter};
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::{blanket::BlanketRunner, options::DagOptions, Runner};
use timed::timed;

use crate::statics::{_get_default_edges, _get_default_tasks, _get_hash, _get_options};

pub mod catchup;
pub mod check_timeout;
pub mod redis_runner;
pub mod routes;
pub mod scheduler;
pub mod statics;

pub fn get_dags_dir() -> String {
    env::var("DAGS_DIR")
        .unwrap_or("./bin".to_string())
        .to_string()
}

fn get_redis_url() -> String {
    env::var("REDIS_URL")
        .unwrap_or("redis://0.0.0.0:6379".to_string())
        .to_string()
}

pub fn _get_dag_path_by_name(dag_name: &str) -> Option<PathBuf> {
    let dags_dir = &get_dags_dir();
    let path: PathBuf = [dags_dir, dag_name].iter().collect();

    if !path.exists() {
        return None;
    }

    Some(path)
}

#[timed(duration(printer = "debug!"))]
pub fn _get_all_tasks(run_id: usize, pool: Pool) -> Vec<Task> {
    RedisRunner::dummy(pool).get_all_tasks(run_id)
}

#[timed(duration(printer = "debug!"))]
pub fn _get_task(run_id: usize, task_id: usize, pool: Pool) -> Task {
    RedisRunner::dummy(pool).get_task_by_id(run_id, task_id)
}

#[timed(duration(printer = "debug!"))]
pub async fn _get_all_task_results(run_id: usize, task_id: usize, pool: Pool) -> Vec<TaskResult> {
    RedisRunner::get_all_results(run_id, task_id, pool).await
}

#[timed(duration(printer = "debug!"))]
pub fn _get_task_status(run_id: usize, task_id: usize, pool: Pool) -> TaskStatus {
    RedisRunner::dummy(pool).get_task_status(run_id, task_id)
}

#[timed(duration(printer = "debug!"))]
pub fn _get_run_status(run_id: usize, pool: Pool) -> i32 {
    RedisRunner::dummy(pool).get_run_status(run_id)
}

#[timed(duration(printer = "debug!"))]
pub fn _get_task_result(run_id: usize, task_id: usize, pool: Pool) -> TaskResult {
    RedisRunner::dummy(pool).get_task_result(run_id, task_id)
}

// TODO cache response to prevent disk read
#[timed(duration(printer = "debug!"))]
pub fn _get_dags() -> Vec<String> {
    let paths: Vec<PathBuf> = match fs::read_dir(get_dags_dir()) {
        Err(e) if e.kind() == ErrorKind::NotFound => vec![],
        Err(e) => panic!("Unexpected Error! {:?}", e),
        Ok(entries) => entries
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect(),
    };

    paths
        .iter()
        .map(|p| {
            p.file_name()
                .and_then(|os_str| os_str.to_str())
                .unwrap()
                .to_string()
        })
        .collect()
}

#[timed(duration(printer = "debug!"))]
pub async fn _trigger_run(dag_name: &str, logical_date: DateTime<Utc>, pool: Pool) -> usize {
    let hash = _get_hash(dag_name);

    // TODO handle error
    let nodes = _get_default_tasks(dag_name).unwrap();
    let edges = _get_default_edges(dag_name).unwrap();

    RedisRunner::from(dag_name, nodes.clone(), edges.clone(), pool.clone()).enqueue_run(
        dag_name,
        &hash,
        logical_date,
    )
}

// #[timed(duration(printer = "debug!"))]
pub fn get_redis_pool() -> Pool {
    let cfg = Config::from_url(get_redis_url());
    cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
}

// #[macro_export]
// macro_rules! transaction_async {
//     ($conn:expr, $keys:expr, $body:expr) => {
//         loop {
//             redis::cmd("WATCH")
//                 .arg($keys)
//                 .query_async::<_, String>($conn)
//                 .await
//                 .unwrap();

//             if let Some(response) = $body {
//                 redis::cmd("UNWATCH")
//                     .query_async::<_, String>($conn)
//                     .await
//                     .unwrap();
//                 break response;
//             }
//         }
//     };
// }

pub fn _get_next_run(options: &DagOptions) -> Vec<Value> {
    if let Some(schedule) = &options.schedule {
        match schedule.parse::<Cron>() {
            Ok(cron) => {
                if !cron.any() {
                    info!("Cron will never match any given time!");
                    return vec![];
                }

                if let Some(start_date) = options.start_date {
                    info!("Start date: {start_date}");
                } else {
                    info!("Start date: None");
                }

                info!("Upcoming:");
                let futures =
                    cron.clone()
                        .iter_from(if let Some(start_date) = options.start_date {
                            if options.should_catchup || start_date > Utc::now() {
                                start_date.into()
                            } else {
                                Utc::now()
                            }
                        } else {
                            Utc::now()
                        });
                let mut next_runs = vec![];
                for time in futures.take(1) {
                    if !cron.contains(time) {
                        info!("Failed check! Cron does not contain {}.", time);
                        break;
                    }
                    if let Some(end_date) = options.end_date {
                        if time > end_date {
                            break;
                        }
                    }
                    next_runs.push(json!({
                        "date": format!("{}", time.format("%F %R"))
                    }));
                    info!("  {}", time.format("%F %R"));
                }

                return next_runs;
            }
            Err(err) => info!("{err}: {schedule}"),
        }
    }

    vec![]
}

pub async fn _get_last_run(dag_name: &str, pool: Pool) -> Vec<Run> {
    let r = RedisRunner::get_last_run(dag_name, pool).await;

    match r {
        Some(run) => vec![run],
        None => vec![],
    }
}

pub async fn _get_recent_runs(dag_name: &str, pool: Pool) -> Vec<Run> {
    RedisRunner::get_recent_runs(dag_name, pool).await
}

pub async fn _trigger_run_from_schedules(
    dag_name: &str,
    server_start_date: DateTime<Utc>,
    cron: &Cron,
    logical_dates: CronTimesIter,
    end_date: Option<DateTime<FixedOffset>>,
    pool: Pool,
) {
    for logical_date in logical_dates {
        if !cron.contains(logical_date) {
            println!("Failed check! Cron does not contain {}.", logical_date);
            break;
        }
        if logical_date >= server_start_date {
            break;
        }
        if let Some(end_date) = end_date {
            if logical_date > end_date {
                break;
            }
        }
        // check if date is already in db
        if RedisRunner::contains_logical_date(
            dag_name,
            &_get_hash(dag_name),
            logical_date,
            pool.clone(),
        )
        .await
        {
            continue;
        }

        _trigger_run(dag_name, logical_date, pool.clone()).await;
        println!(
            "scheduling catchup {dag_name} {}",
            logical_date.format("%F %R")
        );
    }
}

fn _get_schedules_for_catchup(
    cron: &Cron,
    start_date: Option<DateTime<FixedOffset>>,
    should_catchup: bool,
    server_start_date: DateTime<Utc>,
) -> CronTimesIter {
    cron.clone()
        .iter_from(if let Some(start_date) = start_date {
            if should_catchup {
                start_date.into()
            } else {
                server_start_date
            }
        } else {
            server_start_date
        })
}

pub fn tpt_installed() -> bool {
    !matches!(
        String::from_utf8_lossy(&Command::new("which").arg("tpt").output().unwrap().stdout)
            .to_string()
            .as_str()
            .trim(),
        ""
    )
}

#[cfg(test)]

mod tests2 {
    use crate::tpt_installed;

    #[test]
    fn installed() {
        assert!(tpt_installed());
    }
}
