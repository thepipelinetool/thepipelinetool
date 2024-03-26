//! # thepipelinetool
//!
//! `thepipelinetool` organizes your Rust functions into a [Directed Acyclic Graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph) (DAG) structure, ensuring orderly execution according to their dependencies.
//! The DAG is compiled into a CLI executable, which can then be used to list tasks/edges, run individual functions, and execute locally. Finally, deploy to [thepipelinetool_server](https://github.com/thepipelinetool/thepipelinetool_server) to enjoy scheduling, catchup, retries, and live task monitoring with a modern web UI.
pub mod builder;
pub mod cli;
pub mod flow;
pub mod functions;
pub mod hash;
pub mod operator_overrides;
pub mod operators;
pub mod statics;

pub use statics::*;

pub mod prelude {
    pub use crate::cli::parse_cli;
    pub use crate::{
        // builder::TaskBuilder,
        functions::{add_command, add_task, add_task_with_ref, branch, expand, expand_lazy},
    };
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use thepipelinetool_proc_macro::dag;
    pub use thepipelinetool_runner::in_memory::InMemoryRunner;
    pub use thepipelinetool_runner::{blanket::BlanketRunner, Runner};
    pub use thepipelinetool_task::branch::Branch;
    pub use thepipelinetool_task::ordered_queued_task::OrderedQueuedTask;
    pub use thepipelinetool_task::queued_task::QueuedTask;
    pub use thepipelinetool_task::task_options::TaskOptions;
    pub use thepipelinetool_task::task_result::TaskResult;
    pub use thepipelinetool_task::task_status::TaskStatus;
    pub use thepipelinetool_task::trigger_rules::TriggerRules;
    pub use thepipelinetool_task::Task;
    pub use thepipelinetool_utils::execute_function_using_json_files;
}

use serde::Serialize;
use thepipelinetool_task::task_ref_inner::TaskRefInner;

pub struct TaskRef<T: Serialize>(TaskRefInner<T>);
