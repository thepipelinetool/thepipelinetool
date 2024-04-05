use crate::dev::*;
use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};

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

pub fn function_with_name_exists(task_name: &str) -> bool {
    get_functions()
        .read()
        .unwrap()
        .contains_key(task_name)
}


pub fn get_id_by_task_name(name: &str) -> usize {
    get_tasks()
        .read()
        .unwrap()
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("missing task {name}"))
        .id
}