use std::{env, process::Command};

use serde_json::json;
use thepipelinetool_runner::get_tpt_executor_command;
use thepipelinetool_utils::get_default_max_parallelism;

use crate::Executor;
use anyhow::Result;

pub fn tpt_installed() -> Result<bool> {
    Ok(!matches!(
        String::from_utf8_lossy(
            &Command::new("which")
                .arg(get_tpt_command())
                .output()?
                .stdout
        )
        .to_string()
        .as_str()
        .trim(),
        ""
    ))
}

pub fn tpt_executor_installed() -> Result<bool> {
    Ok(!matches!(
        String::from_utf8_lossy(
            &Command::new("which")
                .arg(get_tpt_executor_command())
                .output()?
                .stdout
        )
        .to_string()
        .as_str()
        .trim(),
        ""
    ))
}

const DEFAULT_TPT_COMMAND: &str = "tpt";

pub fn get_tpt_command() -> String {
    env::var("TPT_CMD")
        .unwrap_or(DEFAULT_TPT_COMMAND.to_string())
        .to_string()
}

pub fn get_max_parallelism() -> Result<usize> {
    Ok(env::var("MAX_PARALLELISM")
        .unwrap_or(get_default_max_parallelism().to_string())
        .to_string()
        .parse::<usize>()?)
}

pub fn get_executor_type() -> Result<Executor> {
    Ok(serde_json::from_str(
        &env::var("EXECUTOR")
            .unwrap_or(serde_json::to_string(&json!(Executor::Local)).expect(""))
            .to_string(),
    )?)
}

pub fn get_redis_url() -> String {
    env::var("REDIS_URL")
        .unwrap_or("redis://0.0.0.0:6379".to_string())
        .to_string()
}

pub fn get_check_timeout_loop_interval() -> Result<u64> {
    Ok(env::var("CHECK_TIMEOUT_LOOP_INTERVAL")
        .unwrap_or(5.to_string())
        .parse::<u64>()?)
}

pub fn get_scheduler_loop_interval() -> Result<u64> {
    Ok(env::var("SCHEDULER_LOOP_INTERVAL")
        .unwrap_or(5.to_string())
        .parse::<u64>()?)
}

pub fn get_worker_loop_interval() -> Result<u64> {
    Ok(env::var("WORKER_LOOP_INTERVAL")
        .unwrap_or(1.to_string())
        .parse::<u64>()?)
}

pub fn get_executor_image() -> Result<String> {
    Ok(env::var("EXECUTOR_IMAGE").unwrap_or("executor".to_string()))
}
