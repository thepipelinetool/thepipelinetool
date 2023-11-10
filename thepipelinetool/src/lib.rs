mod cli;
mod dag;
mod options;

pub mod prelude {
    pub use crate::cli::*;
    pub use crate::dag::DAG;
    pub use crate::options::*;
    pub use runner::local::{hash_dag, LocalRunner};
    pub use runner::{DefRunner, Runner};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use task::task::Task;
    pub use task::task_options::TaskOptions;
    pub use task::task_result::TaskResult;
    pub use task::task_status::TaskStatus;
    pub use task::Branch;
    pub use utils::execute_function;
}
