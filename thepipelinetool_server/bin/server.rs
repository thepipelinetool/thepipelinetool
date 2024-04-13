use axum::{http::Method, Router};
use std::path::PathBuf;
// use thepipelinetool_server::catchup::catchup;
use thepipelinetool_server::check_timeout::check_timeout;
use thepipelinetool_server::env::{tpt_executor_installed, tpt_installed};
use thepipelinetool_server::{get_redis_pool, routes::*, scheduler::scheduler};
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use anyhow::Result;
use axum::routing::{get, post};

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    assert!(tpt_installed()?);
    assert!(tpt_executor_installed()?);

    println!("connecting to redis...");
    let pool = get_redis_pool()?;

    // let now = Utc::now();

    // println!("spawning catchup...");
    // {
    //     let pool = pool.clone();
    //     tokio::spawn(async move { catchup(now, pool).await });
    // }

    println!("spawning scheduler...");
    {
        let pool = pool.clone();
        tokio::spawn(async move { scheduler(pool).await });
    }

    println!("spawning check_timeout...");
    {
        let pool = pool.clone();
        tokio::spawn(async move { check_timeout(pool).await });
    }

    let app = Router::new()
        .nest_service("/", ServeDir::new(PathBuf::from("static")))
        .route("/ping", get(ping))
        .route("/pipeline", get(get_pipelines))
        .route("/runs/:pipeline_name", get(get_runs))
        .route("/runs/next/:pipeline_name", get(get_next_run))
        .route("/runs/last/:pipeline_name", get(get_last_run))
        .route("/runs/recent/:pipeline_name", get(get_recent_runs)) // TODO change to recent results?
        .route("/runs/all/:pipeline_name", get(get_runs_with_tasks))
        .route(
            "/trigger/:pipeline_name",
            get(trigger).post(trigger_with_params),
        )
        .route("/statuses/:run_id", get(get_run_status))
        .route("/statuses/:run_id/:task_id", get(get_task_status))
        .route("/results/:run_id/:task_id", get(get_task_result))
        .route("/results/all/:run_id/:task_id", get(get_all_results))
        .route("/logs/:run_id/:task_id/:attempt", get(get_task_log))
        .route("/tasks/:run_id", get(get_all_tasks_by_run_id))
        .route("/tasks/:run_id/:task_id", get(get_task_by_id))
        .route("/tasks/default/:pipeline_name", get(get_default_tasks))
        .route(
            "/tasks/default/:pipeline_name/:task_id",
            get(get_default_task_by_id),
        )
        .route("/graphs/:run_id", get(get_run_graph))
        .route("/graphs/default/:pipeline_name", get(get_default_graph))
        .route("/upload", post(upload_pipeline))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(pool);

    let bind_address = "0.0.0.0:8000";

    println!("Running tpt server on {bind_address}");

    let listener = TcpListener::bind(bind_address).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
