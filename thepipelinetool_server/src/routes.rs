use std::{collections::HashMap, str::from_utf8};

use axum::{
    extract::{self, Path, State},
    http::StatusCode,
    Json,
};

use anyhow::Result;

use thepipelinetool_core::dev::*;

use crate::{statics::_get_options, *};

pub async fn ping() -> &'static str {
    "pong"
}

// TODO paginate

pub async fn get_runs(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(json!(RedisBackend::get_runs(&dag_name, pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })?
        .iter()
        .map(|r| json!({
            "run_id": r.run_id.to_string(),
            "date": r.scheduled_date_for_dag_run,
        }))
        .collect::<Vec<Value>>())
    .into())
}

pub async fn get_next_run(
    Path(dag_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // TODO handle error
    let options = _get_options(&dag_name).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?;

    Ok(json!(_get_next_run(&options)).into())
}

pub async fn get_last_run(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(json!(_get_last_run(&dag_name, pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?)
    .into())
}

pub async fn get_recent_runs(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(json!(_get_recent_runs(&dag_name, pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?)
    .into())
}

// TODO return only statuses?
pub async fn get_runs_with_tasks(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut res = json!({});

    for run in RedisBackend::get_runs(&dag_name, pool.clone())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })?
        .iter()
    {
        let mut tasks = json!({});
        for task in _get_all_tasks(run.run_id, pool.clone()).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })? {
            tasks[format!("{}_{}", task.name, task.id)] = json!(task);
        }
        res[run.run_id.to_string()] = json!({
            "date": run.scheduled_date_for_dag_run,
            "tasks": tasks,
        });
    }
    Ok(res.into())
}

pub async fn get_default_tasks(
    Path(dag_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(
        serde_json::to_value(_get_default_tasks(&dag_name).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })?)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })?
        .into(),
    )
}

pub async fn get_default_task(
    Path((dag_name, task_id)): Path<(String, usize)>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // TODO handle error

    let default_tasks = _get_default_tasks(&dag_name).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?;

    for t in default_tasks {
        if t.id == task_id {
            return Ok(json!(t).into());
        }
    }

    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "could not get run graph for pipeline with name: {:?}",
            dag_name
        ),
    ))
}

pub async fn get_all_tasks(
    Path(run_id): Path<usize>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(json!(_get_all_tasks(run_id, pool).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?)
    .into())
}

pub async fn get_task(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(json!(_get_task(run_id, task_id, pool).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?)
    .into())
}

pub async fn get_all_task_results(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(json!(_get_all_task_results(run_id, task_id, pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })?)
    .into())
}

pub async fn get_task_status(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> Result<String, (StatusCode, String)> {
    Ok(from_utf8(&[_get_task_status(run_id, task_id, pool)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })?
        .as_u8()])
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?
    .to_owned())
}

pub async fn get_run_status(
    Path(run_id): Path<usize>,
    State(pool): State<Pool>,
) -> Result<String, (StatusCode, String)> {
    Ok(from_utf8(&[
        match _get_run_status(run_id, pool).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })? {
            0 => 0,
            -1 => 1,
            a => a as u8,
        },
    ])
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?
    .to_owned())
}

pub async fn get_task_result(
    Path((run_id, task_id)): Path<(usize, usize)>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    Ok(json!(_get_task_result(run_id, task_id, pool).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?)
    .into())
}

pub async fn get_task_log(
    Path((run_id, task_id, attempt)): Path<(usize, usize, usize)>,
    State(pool): State<Pool>,
) -> Result<String, (StatusCode, String)> {
    RedisBackend::dummy(pool)
        .get_log(run_id, task_id, attempt)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })
}

pub async fn get_dags(State(pool): State<Pool>) -> Result<Json<Value>, (StatusCode, String)> {
    let mut result: Vec<Value> = vec![];

    for dag_name in _get_dags().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })? {
        let options = _get_options(&dag_name).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not get run graph for pipeline with name: {:?}", e),
            )
        })?;
        result.push(json!({
            "last_run": _get_last_run(&dag_name, pool.clone()).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("could not get run graph for pipeline with name: {:?}", e),
                )
            })?,
            "next_run":_get_next_run(&options),
            "options": &options,
            "dag_name": &dag_name,
        }));
    }

    Ok(json!(result).into())
}

pub async fn get_run_graph(
    Path(run_id): Path<usize>,
    State(pool): State<Pool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let backend = RedisBackend::dummy(pool);
    let tasks = backend.get_all_tasks(run_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not get run graph for pipeline with name: {:?}", e),
        )
    })?;
    let mut task_statuses: Vec<(usize, String, TaskStatus)> = vec![];

    for task in &tasks {
        task_statuses.push((
            task.id,
            task.name.clone(),
            backend.get_task_status(run_id, task.id).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("could not get run graph for pipeline with name: {:?}", e),
                )
            })?,
        ));
    }

    let mut downstream_ids: HashMap<usize, Vec<usize>> = HashMap::new();

    for t in tasks {
        downstream_ids.insert(
            t.id,
            backend.get_downstream(run_id, t.id).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("could not get_downstream for pipeline with name: {:?}", e),
                )
            })?,
        );
    }

    Ok(json!(get_graphite_graph(&task_statuses, &downstream_ids)).into())
}

// (StatusCode, String) for Error {
//     fn into_response(self) -> axum::response::Response {
//         todo!()
//     }
// }

pub async fn get_default_graph(
    Path(dag_name): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let tasks = _get_default_tasks(&dag_name); // TODO handle missing dag error
    let edges = _get_default_edges(&dag_name);

    match (tasks, edges) {
        (Ok(tasks), Ok(edges)) => Ok(json!(get_default_graphite_graph(&tasks, &edges)).into()),
        e => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not trigger pipeline with name: {:?}", e),
        )),
    }
}

pub async fn trigger(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
) -> Result<Json<usize>, (StatusCode, String)> {
    let tasks = _get_default_tasks(&dag_name);
    let edges = _get_default_edges(&dag_name);
    let hash = _get_hash(&dag_name);
    match (tasks, edges, hash) {
        (Ok(tasks), Ok(edges), Ok(hash)) => {
            let scheduled_date = Utc::now();
            let mut backend = RedisBackend::from(tasks, edges, pool.clone());
            let run_id = backend
                .create_new_run(&dag_name, &hash, scheduled_date)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("could not trigger pipeline with name: {:?}", e),
                    )
                })?;

            tokio::spawn(async move {
                backend.enqueue_run(run_id, &dag_name, &hash, scheduled_date, None)
            });

            Ok(run_id.into())
        }
        e => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not trigger pipeline with name: {:?}", e),
        )),
    }
}

pub async fn trigger_with_params(
    Path(dag_name): Path<String>,
    State(pool): State<Pool>,
    extract::Json(params): extract::Json<Value>,
) -> Result<Json<usize>, (StatusCode, String)> {
    let tasks = _get_default_tasks(&dag_name); // TODO handle missing dag error
    let edges = _get_default_edges(&dag_name);
    let hash = _get_hash(&dag_name);
    match (tasks, edges, hash) {
        (Ok(tasks), Ok(edges), Ok(hash)) => {
            let scheduled_date = Utc::now();
            let mut backend = RedisBackend::from(tasks, edges, pool.clone());
            let run_id = backend
                .create_new_run(&dag_name, &hash, scheduled_date)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("could not trigger pipeline with name: {:?}", e),
                    )
                })?;

            tokio::spawn(async move {
                backend.enqueue_run(run_id, &dag_name, &hash, scheduled_date, Some(params))
            });

            Ok(run_id.into())
        }
        e => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not trigger pipeline with name: {:?}", e),
        )),
    }
}
