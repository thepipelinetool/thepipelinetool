use std::env;

use backend::Backend;

pub mod backend;
pub mod blanket_backend;
pub mod in_memory_backend;
pub mod pipeline;
pub mod pipeline_options;
pub mod run;

const DEFAULT_TPT_X_COMMAND: &str = "tpt_executor";

pub fn get_tpt_executor_command() -> String {
    env::var("TPT_X_CMD")
        .unwrap_or(DEFAULT_TPT_X_COMMAND.to_string())
        .to_string()
}
