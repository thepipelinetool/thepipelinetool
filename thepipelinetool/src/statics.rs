use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};
use thepipelinetool_task::Task;

type StaticTasks = RwLock<Vec<Task>>;
type StaticFunctions = RwLock<HashMap<String, Box<dyn Fn(Value) -> Value + Sync + Send>>>;
type StaticEdges = RwLock<HashSet<(usize, usize)>>;

static TASKS: OnceLock<StaticTasks> = OnceLock::new();
static FUNCTIONS: OnceLock<StaticFunctions> = OnceLock::new();
static EDGES: OnceLock<StaticEdges> = OnceLock::new();

pub fn get_tasks() -> &'static StaticTasks {
    TASKS.get_or_init(StaticTasks::default)
}

pub fn get_functions() -> &'static StaticFunctions {
    FUNCTIONS.get_or_init(StaticFunctions::default)
}

pub fn get_edges() -> &'static StaticEdges {
    EDGES.get_or_init(StaticEdges::default)
}
