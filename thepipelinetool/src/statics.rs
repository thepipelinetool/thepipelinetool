use crate::dev::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock};
use thepipelinetool_operators::bash::bash_operator;

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
    // load built-in operators

    FUNCTIONS.get_or_init(|| {
        let functions: RwLock<HashMap<String, Box<dyn Fn(Value) -> Value + Sync + Send>>> =
            RwLock::new(HashMap::new());

        for operator in vec![bash_operator] {
            let function_name = function_name_as_string(&operator).to_string();
            let wrapped_function = wrap_function(operator);

            functions
                .write()
                .unwrap()
                .insert(function_name, Box::new(wrapped_function));
        }
        functions
    })
}

pub fn get_edges() -> &'static StaticEdges {
    EDGES.get_or_init(StaticEdges::default)
}
