//! # thepipelinetool
//!
//! `thepipelinetool` organizes your Rust functions into a [Directed Acyclic Graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph) (DAG) structure, ensuring orderly execution according to their dependencies.
//! The DAG is compiled into a CLI executable, which can then be used to list tasks/edges, run individual functions, and execute locally. Finally, deploy to [thepipelinetool_server](https://github.com/thepipelinetool/thepipelinetool_server) to enjoy scheduling, catchup, retries, and live task monitoring with a modern web UI.
mod cli;
mod flow;
mod functions;
mod hash;
mod operators;
mod ops_overrides;
mod statics;

use serde::Serialize;
pub use thepipelinetool_task::task_ref_inner::TaskRefInner;

pub struct TaskRef<T: Serialize>(TaskRefInner<T>);

pub mod prelude {
    pub use crate::cli::parse_cli;
    pub use crate::operators::*;
    pub use crate::{functions::*, TaskRef, TaskRefInner};

    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use thepipelinetool_proc_macro::dag;
    pub use thepipelinetool_task::branch::Branch;
    pub use thepipelinetool_task::task_options::TaskOptions;
}

pub mod dev {
    pub use crate::cli::*;
    pub use crate::hash::hash_dag;
    pub use crate::prelude::*;
    pub use crate::statics::*;
    pub use thepipelinetool_runner::in_memory::{run_in_memory, InMemoryRunner};
    pub use thepipelinetool_runner::{blanket::BlanketRunner, Runner};
    pub use thepipelinetool_task::ordered_queued_task::OrderedQueuedTask;
    pub use thepipelinetool_task::queued_task::QueuedTask;
    pub use thepipelinetool_task::task_result::TaskResult;
    pub use thepipelinetool_task::task_status::TaskStatus;
    pub use thepipelinetool_task::Task;
    pub use thepipelinetool_utils::*;
}
