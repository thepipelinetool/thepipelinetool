use axum::{http::Method, Router};
use chrono::Utc;
use std::path::PathBuf;
use thepipelinetool_server::catchup::spawn_catchup;
use thepipelinetool_server::check_timeout::spawn_check_timeout;
use thepipelinetool_server::scheduler::spawn_scheduler;
use thepipelinetool_server::{get_redis_pool, routes::*, tpt_installed};
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use axum::routing::get;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    assert!(tpt_installed());

    println!("connecting to redis...");
    let pool = get_redis_pool();

    let now = Utc::now();

    println!("spawning catchup...");
    spawn_catchup(now, pool.clone());

    println!("spawning scheduler...");
    spawn_scheduler(now, pool.clone());

    println!("spawning check_timeout...");
    spawn_check_timeout(pool.clone());

    let app = Router::new()
        .nest_service("/", ServeDir::new(PathBuf::from("static")))
        .route("/ping", get(ping))
        .route("/dags", get(get_dags))
        .route("/runs/:dag_name", get(get_runs))
        .route("/runs/next/:dag_name", get(get_next_run))
        .route("/runs/last/:dag_name", get(get_last_run))
        .route("/runs/recent/:dag_name", get(get_recent_runs)) // TODO change to recent results?
        .route("/runs/all/:dag_name", get(get_runs_with_tasks))
        .route("/trigger/:dag_name", get(trigger))
        .route("/statuses/:run_id", get(get_run_status))
        .route("/statuses/:run_id/:task_id", get(get_task_status))
        .route("/results/:run_id/:task_id", get(get_task_result))
        .route("/results/all/:run_id/:task_id", get(get_all_task_results))
        .route("/logs/:run_id/:task_id/:attempt", get(get_task_log))
        .route("/tasks/:run_id", get(get_all_tasks))
        .route("/tasks/:run_id/:task_id", get(get_task))
        .route("/tasks/default/:dag_name", get(get_default_tasks))
        .route("/tasks/default/:dag_name/:task_id", get(get_default_task))
        .route("/graphs/:run_id", get(get_run_graph))
        .route("/graphs/default/:dag_name", get(get_default_graph))
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

    let listener = TcpListener::bind(bind_address).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
