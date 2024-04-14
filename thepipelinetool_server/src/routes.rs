use std::{collections::HashMap, str::from_utf8};

use axum::{
    extract::{self, Path, State},
    http::StatusCode,
    Json,
};

use chrono::Utc;
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::pipeline::Pipeline;

use crate::*;

type ServerResult<E> = Result<E, (StatusCode, String)>;

fn service_err(s: String) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, s)
}

pub async fn ping() -> &'static str {
    "pong"
}

// TODO paginate

pub async fn get_runs(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    Ok(json!(RedisBackend::get_runs(&pipeline_name, pool)
        .await
        .map_err(|e| service_err(format!(
            "could not get runs for pipeline '{}'\n{:?}",
            pipeline_name, e
        ),))?
        .iter()
        .map(|r| json!({
            "run_id": r.run_id.to_string(),
            "date": r.scheduled_date_for_run,
        }))
        .collect::<Vec<Value>>())
    .into())
}

pub async fn get_next_run(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    // let options = _get_options(&pipeline_name).map_err(|e| {
    //     service_err(format!(
    //         "could not get next run for pipeline '{}'\n{:?}",
    //         pipeline_name, e
    //     ))
    // })?;

    // Ok(json!(_get_next_run(&options)).into())
    todo!()
}

pub async fn get_last_run(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    Ok(json!(_get_last_run(&pipeline_name, pool)
        .await
        .map_err(|e| service_err(format!(
            "could not get runs for pipeline '{}'\n{:?}",
            pipeline_name, e
        ),))?)
    .into())
}

pub async fn get_recent_runs(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    Ok(json!(_get_recent_runs(&pipeline_name, pool)
        .await
        .map_err(|e| service_err(format!(
            "could not get recent runs for pipeline '{}'\n{:?}",
            pipeline_name, e
        ),))?)
    .into())
}

// TODO return only statuses?
pub async fn get_runs_with_tasks(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    let mut res = json!({});

    for run in RedisBackend::get_runs(&pipeline_name, pool.clone())
        .await
        .map_err(|e| {
            service_err(format!(
                "could not get runs with tasks for pipeline '{}'\n{:?}",
                pipeline_name, e
            ))
        })?
        .iter()
    {
        let mut tasks = json!({});
        for task in _get_all_tasks_by_run_id(run.run_id, pool.clone()).map_err(|e| {
            service_err(format!(
                "could not get runs with tasks for pipeline '{}'\n{:?}",
                pipeline_name, e
            ))
        })? {
            tasks[format!("{}_{}", task.name, task.id)] = json!(task);
        }
        res[run.run_id.to_string()] = json!({
            "date": run.scheduled_date_for_run,
            "tasks": tasks,
        });
    }
    Ok(res.into())
}

pub async fn get_default_tasks(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    let backend = RedisBackend::from(&pipeline_name, pool);
    let tasks = backend.get_default_tasks().map_err(|e| {
        service_err(format!(
            "could not get default tasks for pipeline '{}'\n{:?}",
            pipeline_name, e
        ))
    })?;

    Ok(json!(tasks).into())
}

pub async fn get_default_task_by_id(
    Path((pipeline_name, task_id)): Path<(String, usize)>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    let backend = RedisBackend::from(&pipeline_name, pool);
    let default_tasks = backend.get_default_tasks().map_err(|e| {
        service_err(format!(
            "could not get default tasks for pipeline '{}'\n{:?}",
            pipeline_name, e
        ))
    })?;
    for t in default_tasks {
        if t.id == task_id {
            return Ok(json!(t).into());
        }
    }

    Err(service_err(format!(
        "no existing default task_id '{}' for pipeline '{}'",
        task_id, pipeline_name
    )))
}

pub async fn get_all_tasks_by_run_id(
    Path(run_id): Path<usize>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    Ok(json!(
        _get_all_tasks_by_run_id(run_id, pool).map_err(|e| service_err(format!(
            "could not get all tasks for run_id '{}'\n{:?}",
            run_id, e
        ),))?
    )
    .into())
}

pub async fn get_task_by_id(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    Ok(json!(
        _get_task_by_id(run_id, task_id, pool).map_err(|e| service_err(format!(
            "could not get task for task_id '{}' and run_id {}\n{:?}",
            task_id, run_id, e
        ),))?
    )
    .into())
}

pub async fn get_all_results(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    Ok(json!(_get_all_task_results(run_id, task_id, pool)
        .await
        .map_err(|e| service_err(format!(
            "could not get all results for task_id '{}' and run_id '{}'\n{:?}",
            task_id, run_id, e
        ),))?)
    .into())
}

pub async fn get_task_status(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> ServerResult<String> {
    Ok(from_utf8(&[_get_task_status(run_id, task_id, pool)
        .map_err(|e| {
            service_err(format!(
                "could not get task status for run_id '{}' and task_id '{}'\n{:?}",
                run_id, task_id, e
            ))
        })?
        .as_u8()])
    .expect("")
    .to_owned())
}

pub async fn get_run_status(
    Path(run_id): Path<usize>,
    State(pool): State<Pool>,
) -> ServerResult<String> {
    Ok(from_utf8(&[
        match _get_run_status(run_id, pool).map_err(|e| {
            service_err(format!(
                "could not get run status for run_id '{}'\n{:?}",
                run_id, e
            ))
        })? {
            0 => 0,
            -1 => 1,
            a => a as u8,
        },
    ])
    .expect("")
    .to_owned())
}

pub async fn get_task_result(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    Ok(json!(
        _get_task_result(run_id, task_id, pool).map_err(|e| service_err(format!(
            "could not get result for task_id '{}' and run_id '{}'\n{:?}",
            task_id, run_id, e
        ),))?
    )
    .into())
}

pub async fn get_task_log(
    Path((run_id, task_id, attempt)): Path<(usize, usize, usize)>,
    State(pool): State<Pool>,
) -> ServerResult<String> {
    RedisBackend::dummy(pool)
        .get_log(run_id, task_id, attempt)
        .map_err(|e| {
            service_err(format!(
                "could not get task log for run_id '{}', task_id '{}', and attempt '{}'\n{:?}",
                run_id, task_id, attempt, e
            ))
        })
}

pub async fn get_pipelines(State(pool): State<Pool>) -> ServerResult<Json<Value>> {
    Ok(json!(_get_pipelines(pool)
        .await
        .map_err(|e| { service_err(format!("could not get pipelines\n{:?}", e)) })?)
    .into())
}

pub async fn get_run_graph(
    Path(run_id): Path<usize>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    let backend = RedisBackend::dummy(pool);
    let tasks = backend.get_all_tasks(run_id).map_err(|e| {
        service_err(format!(
            "could not get all tasks for run_id '{}'\n{:?}",
            run_id, e
        ))
    })?;
    let mut task_statuses: Vec<(usize, String, TaskStatus)> = vec![];

    for task in &tasks {
        task_statuses.push((
            task.id,
            task.name.clone(),
            backend.get_task_status(run_id, task.id).map_err(|e| {
                service_err(format!(
                    "could not get task status for run_id '{}' and task_id '{}'\n{:?}",
                    run_id, task.id, e
                ))
            })?,
        ));
    }

    let mut downstream_ids: HashMap<usize, Vec<usize>> = HashMap::new();

    for t in tasks {
        downstream_ids.insert(
            t.id,
            backend.get_downstream(run_id, t.id).map_err(|e| {
                service_err(format!(
                    "could not get downstream for run_id '{}' and task_id '{}'\n{:?}",
                    run_id, t.id, e
                ))
            })?,
        );
    }

    Ok(json!(get_graphite_graph(&task_statuses, &downstream_ids)).into())
}

// ServerResponse for Error {
//     fn into_response(self) -> axum::response::Response {
//         todo!()
//     }
// }

pub async fn get_default_graph(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<Value>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    let backend = RedisBackend::from(&pipeline_name, pool);
    let tasks = backend.get_default_tasks().map_err(|e| {
        service_err(format!(
            "could not get default tasks for pipeline '{}'\n{:?}",
            pipeline_name, e
        ))
    })?;
    let edges = backend.get_default_edges().map_err(|e| {
        service_err(format!(
            "could not get default edges for pipeline '{}'\n{:?}",
            pipeline_name, e
        ))
    })?;

    Ok(json!(get_default_graphite_graph(&tasks, &edges)).into())
}

pub async fn assert_pipeline_exists(pipeline_name: &str, pool: Pool) -> ServerResult<()> {
    if !_get_pipelines(pool)
        .await
        .map_err(|e| service_err(format!("could get pipelines\n{:?}", e)))?
        .contains(pipeline_name)
    {
        Err(service_err(format!(
            "could not find pipeline '{}'\n",
            pipeline_name
        )))
    } else {
        Ok(())
    }
}

pub async fn trigger(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
) -> ServerResult<Json<usize>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    let scheduled_date = Utc::now();
    let mut backend = RedisBackend::from(&pipeline_name, pool.clone());
    let run = backend.create_new_run(scheduled_date).map_err(|e| {
        service_err(format!(
            "could not create new run for pipeline '{}'\n{:?}",
            pipeline_name, e
        ))
    })?;
    let run_id = run.run_id;

    tokio::spawn(async move { backend.enqueue_run(&run, None) });

    Ok(run_id.into())
}

pub async fn trigger_with_params(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
    extract::Json(params): extract::Json<Value>,
) -> ServerResult<Json<usize>> {
    assert_pipeline_exists(&pipeline_name, pool.clone()).await?;

    let scheduled_date = Utc::now();
    let mut backend = RedisBackend::from(&pipeline_name, pool.clone());
    let run = backend.create_new_run(scheduled_date).map_err(|e| {
        service_err(format!(
            "could not create new run for pipeline '{}'\n{:?}",
            pipeline_name, e
        ))
    })?;
    let run_id = run.run_id;

    tokio::spawn(async move { backend.enqueue_run(&run, Some(params)) });

    Ok(run_id.into())
}

pub async fn upload_pipeline(
    Path(pipeline_name): Path<String>,
    State(pool): State<Pool>,
    extract::Json(pipeline): extract::Json<Pipeline>,
) -> ServerResult<String> {
    RedisBackend::upload_pipeline(&pipeline, &pipeline_name, pool.clone())
        .await
        .map_err(|e| {
            service_err(format!(
                "could not upload pipeline '{}'\n{:?}",
                pipeline_name, e
            ))
        })?;
    Ok("ok".to_string())
}
