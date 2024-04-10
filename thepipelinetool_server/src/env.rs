use std::{env, process::Command};

use serde_json::json;
use thepipelinetool_runner::get_tpt_executor_command;
use thepipelinetool_utils::get_default_max_parallelism;

use crate::Executor;

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
