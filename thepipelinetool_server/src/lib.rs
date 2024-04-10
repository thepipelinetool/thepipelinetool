use std::{fs, io::ErrorKind, path::PathBuf};

use chrono::{DateTime, Utc};
use deadpool::Runtime;
use deadpool_redis::{Config, Pool};
use env::get_redis_url;
use log::info;
use redis_backend::{RedisBackend, Run};
use saffron::{Cron, CronTimesIter};
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::{
    backend::Backend, blanket_backend::BlanketBackend, get_dags_dir, options::DagOptions,
};

use crate::statics::{_get_default_edges, _get_default_tasks, _get_hash};

pub mod catchup;
pub mod check_timeout;
pub mod env;
pub mod redis_backend;
pub mod routes;
pub mod scheduler;
pub mod statics;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Executor {
    Local,
    Docker,
    Kubernetes,
}

pub fn _get_all_tasks(run_id: usize, pool: Pool) -> Vec<Task> {
    RedisBackend::dummy(pool).get_all_tasks(run_id)
}

pub fn _get_task(run_id: usize, task_id: usize, pool: Pool) -> Task {
    RedisBackend::dummy(pool).get_task_by_id(run_id, task_id)
}

pub async fn _get_all_task_results(run_id: usize, task_id: usize, pool: Pool) -> Vec<TaskResult> {
    RedisBackend::get_all_results(run_id, task_id, pool).await
}

pub fn _get_task_status(run_id: usize, task_id: usize, pool: Pool) -> TaskStatus {
    RedisBackend::dummy(pool).get_task_status(run_id, task_id)
}

pub fn _get_run_status(run_id: usize, pool: Pool) -> i32 {
    RedisBackend::dummy(pool).get_run_status(run_id)
}

pub fn _get_task_result(run_id: usize, task_id: usize, pool: Pool) -> TaskResult {
    RedisBackend::dummy(pool).get_task_result(run_id, task_id)
}

// TODO cache response to prevent disk read

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

// pub async fn _trigger_run<T>(
//     run_id: usize,
//     dag_name: &str,
//     scheduled_date_for_dag_run: DateTime<Utc>,
//     pool: Pool,
//     trigger_params: Option<Value>,
//     mut backend: T,
// ) where
//     T: BlanketBackend,
// {
//     let hash = _get_hash(dag_name);
//     backend.enqueue_run(
//         run_id,
//         dag_name,
//         &hash,
//         scheduled_date_for_dag_run,
//         trigger_params,
//     )
// }

//
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
                let futures = cron.clone().iter_from(
                    if let Some(start_date) = options.get_start_date_with_timezone() {
                        if options.should_catchup || start_date > Utc::now() {
                            start_date
                        } else {
                            Utc::now()
                        }
                    } else {
                        Utc::now()
                    },
                );
                let mut next_runs = vec![];
                for time in futures.take(1) {
                    if !cron.contains(time) {
                        info!("Failed check! Cron does not contain {}.", time);
                        break;
                    }
                    if let Some(end_date) = options.get_end_date_with_timezone() {
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
    let r = RedisBackend::get_last_run(dag_name, pool).await;

    match r {
        Some(run) => vec![run],
        None => vec![],
    }
}

pub async fn _get_recent_runs(dag_name: &str, pool: Pool) -> Vec<Run> {
    RedisBackend::get_recent_runs(dag_name, pool).await
}

pub async fn _trigger_run_from_schedules(
    dag_name: &str,
    server_start_date: DateTime<Utc>,
    cron: &Cron,
    scheduled_dates: CronTimesIter,
    end_date: Option<DateTime<Utc>>,
    pool: Pool,
) {
    for scheduled_date in scheduled_dates {
        if !cron.contains(scheduled_date) {
            println!("Failed check! Cron does not contain {}.", scheduled_date);
            break;
        }
        if scheduled_date >= server_start_date {
            break;
        }
        if let Some(end_date) = end_date {
            if scheduled_date > end_date {
                break;
            }
        }
        // check if date is already in db
        if RedisBackend::contains_logical_date(
            dag_name,
            &_get_hash(dag_name),
            scheduled_date,
            pool.clone(),
        )
        .await
        {
            continue;
        }
        let nodes = _get_default_tasks(dag_name).unwrap(); // TODO handle missing dag error
        let edges = _get_default_edges(dag_name).unwrap();
        let hash = _get_hash(dag_name);

        let mut backend = RedisBackend::from(nodes, edges, pool.clone());
        let run_id = backend.create_new_run(dag_name, &hash, scheduled_date);
        backend.enqueue_run(run_id, dag_name, &hash, scheduled_date, None);
        println!(
            "scheduling catchup {dag_name} {}",
            scheduled_date.format("%F %R")
        );
    }
}