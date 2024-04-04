use std::str::from_utf8;

use axum::{
    extract::{Path, State},
    Json,
};

use thepipelinetool_core::dev::*;
use thepipelinetool_runner::in_memory::InMemoryRunner;
use timed::timed;

use crate::*;

#[timed(duration(printer = "debug!"))]
pub async fn ping() -> &'static str {
    "pong"
}

// TODO paginate
#[timed(duration(printer = "debug!"))]
pub async fn get_runs(Path(dag_name): Path<String>, State(pool): State<Pool>) -> Json<Value> {
    json!(RedisRunner::get_runs(&dag_name, pool)
        .await
        .iter()
        .map(|r| json!({
            "run_id": r.run_id.to_string(),
            "date": r.date,
        }))
        .collect::<Vec<Value>>())
    .into()
}

#[timed(duration(printer = "debug!"))]
pub async fn get_next_run(Path(dag_name): Path<String>) -> Json<Value> {
    // TODO handle error
    let options = _get_options(&dag_name).unwrap();

    json!(_get_next_run(&options)).into()
}

#[timed(duration(printer = "debug!"))]
pub async fn get_last_run(Path(dag_name): Path<String>, State(pool): State<Pool>) -> Json<Value> {
    json!(_get_last_run(&dag_name, pool).await).into()
}

#[timed(duration(printer = "debug!"))]
pub async fn get_recent_runs(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
) -> Json<Value> {
    json!(_get_recent_runs(&dag_name, pool).await).into()
}

// TODO return only statuses?
pub async fn get_runs_with_tasks(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
) -> Json<Value> {
    let mut res = json!({});

    for run in RedisRunner::get_runs(&dag_name, pool.clone()).await.iter() {
        let mut tasks = json!({});
        for task in _get_all_tasks(run.run_id, pool.clone()) {
            tasks[format!("{}_{}", task.name, task.id)] = json!(task);
        }
        res[run.run_id.to_string()] = json!({
            "date": run.date,
            "tasks": tasks,
        });
    }
    res.into()
}

pub async fn get_default_tasks(Path(dag_name): Path<String>) -> Json<Value> {
    serde_json::to_value(_get_default_tasks(&dag_name))
        .unwrap()
        .into()
}

pub async fn get_default_task(Path((dag_name, task_id)): Path<(String, usize)>) -> Json<Value> {
    // TODO handle error

    let default_tasks = _get_default_tasks(&dag_name).unwrap();

    json!(&default_tasks.iter().find(|t| t.id == task_id).unwrap()).into()
}

pub async fn get_all_tasks(Path(run_id): Path<usize>, State(pool): State<Pool>) -> Json<Value> {
    json!(_get_all_tasks(run_id, pool)).into()
}

pub async fn get_task(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> Json<Value> {
    json!(_get_task(run_id, task_id, pool)).into()
}

pub async fn get_all_task_results(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> Json<Value> {
    json!(_get_all_task_results(run_id, task_id, pool).await).into()
}

pub async fn get_task_status(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> String {
    from_utf8(&[_get_task_status(run_id, task_id, pool).as_u8()])
        .unwrap()
        .to_owned()
}

pub async fn get_run_status(Path(run_id): Path<usize>, State(pool): State<Pool>) -> String {
    from_utf8(&[match _get_run_status(run_id, pool) {
        0 => 0,
        -1 => 1,
        a => a as u8,
    }])
    .unwrap()
    .to_owned()
}

pub async fn get_task_result(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> Json<Value> {
    json!(_get_task_result(run_id, task_id, pool)).into()
}

pub async fn get_task_log(
    Path((run_id, task_id, attempt)): Path<(usize, usize, usize)>,
    State(pool): State<Pool>,
) -> String {
    RedisRunner::dummy(pool).get_log(run_id, task_id, attempt)
}

pub async fn get_dags(State(pool): State<Pool>) -> Json<Value> {
    let mut result: Vec<Value> = vec![];

    for dag_name in _get_dags() {
        result.push(json!({
            "last_run": _get_last_run(&dag_name, pool.clone()).await,
            "next_run":_get_next_run(&_get_options(&dag_name).unwrap()),
            "options":_get_options(&dag_name),
            "dag_name": &dag_name,
        }));
    }

    json!(result).into()
}

pub async fn get_run_graph(Path(run_id): Path<usize>, State(pool): State<Pool>) -> Json<Value> {
    json!(RedisRunner::dummy(pool).get_graphite_graph(run_id)).into()
}

pub async fn get_default_graph(Path(dag_name): Path<String>) -> Json<Value> {
    let nodes = _get_default_tasks(&dag_name);
    let edges = _get_default_edges(&dag_name);

    // TODO handle error
    assert!(nodes.is_some() && edges.is_some());

    let mut runner = InMemoryRunner::new(&nodes.unwrap(), &edges.unwrap());
    runner.enqueue_run("in_memory", "", Utc::now());

    json!(runner.get_graphite_graph(0)).into()
}

pub async fn trigger(Path(dag_name): Path<String>, State(pool): State<Pool>) -> Json<usize> {
    // json!(
    tokio::spawn(async move { _trigger_run(&dag_name, Utc::now(), pool).await })
        .await
        .unwrap()
        .into()
    // )
    // .into()
}
