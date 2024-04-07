use std::{env, path::PathBuf, process::Command};

use serde_json::json;
use thepipelinetool_utils::get_default_max_parallelism;

use crate::local_runner::Executor;

pub fn get_dags_dir() -> String {
    env::var("DAGS_DIR")
        .unwrap_or("./bin".to_string())
        .to_string()
}

pub fn get_dag_path_by_name(dag_name: &str) -> Option<PathBuf> {
    let dags_dir = &get_dags_dir();
    let path: PathBuf = [dags_dir, dag_name].iter().collect();

    if !path.exists() {
        return None;
    }

    Some(path)
}

pub fn tpt_installed() -> bool {
    !matches!(
        String::from_utf8_lossy(
            &Command::new("which")
                .arg(get_tpt_command())
                .output()
                .unwrap()
                .stdout
        )
        .to_string()
        .as_str()
        .trim(),
        ""
    )
}

pub fn tpt_executor_installed() -> bool {
    !matches!(
        String::from_utf8_lossy(
            &Command::new("which")
                .arg(get_tpt_executor_command())
                .output()
                .unwrap()
                .stdout
        )
        .to_string()
        .as_str()
        .trim(),
        ""
    )
}

const DEFAULT_TPT_COMMAND: &str = "tpt";

pub fn get_tpt_command() -> String {
    env::var("TPT_CMD")
        .unwrap_or(DEFAULT_TPT_COMMAND.to_string())
        .to_string()
}

const DEFAULT_TPT_X_COMMAND: &str = "tpt_executor";

pub fn get_tpt_executor_command() -> String {
    env::var("TPT_X_CMD")
        .unwrap_or(DEFAULT_TPT_X_COMMAND.to_string())
        .to_string()
}

pub fn get_max_parallelism() -> usize {
    env::var("MAX_PARALLELISM")
        .unwrap_or(get_default_max_parallelism().to_string())
        .to_string()
        .parse::<usize>()
        .unwrap()
}

pub fn get_executor_type() -> Executor {
    serde_json::from_str(
        &env::var("EXECUTOR")
            .unwrap_or(serde_json::to_string(&json!(Executor::Local)).unwrap())
            .to_string(),
    )
    .unwrap()
}

pub fn get_redis_url() -> String {
    env::var("REDIS_URL")
        .unwrap_or("redis://0.0.0.0:6379".to_string())
        .to_string()
}
