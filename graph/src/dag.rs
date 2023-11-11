use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};
use utils::{collector, function_name_as_string};

use serde_json::Value;
use task::task::Task;

// use crate::options::DagOptions;
pub struct Dag {
    pub nodes: Vec<Task>,
    pub functions: HashMap<String, Box<dyn Fn(Value) -> Value + Sync + Send>>,
    pub edges: HashSet<(usize, usize)>,
    pub options: DagOptions,
}

impl Dag {
    pub fn new() -> Self {
        let mut functions: HashMap<String, Box<dyn Fn(Value) -> Value + Sync + Send>> =
            HashMap::new();
        let function_name = function_name_as_string(&collector).to_string();
        functions.insert(function_name.clone(), Box::new(collector));

        Self {
            nodes: Vec::new(),
            functions,
            edges: HashSet::new(),
            options: DagOptions::default(),
        }
    }
}

use std::sync::OnceLock;

use crate::options::DagOptions;
// static DAG: OnceLock<Dag> = OnceLock::new();
// pub fn get_dag() -> &'static Dag {
//     DAG.get_or_init(|| Dag::new())
// }

static DAG: OnceLock<Mutex<Dag>> = OnceLock::new();

pub fn get_dag() -> &'static Mutex<Dag> {
    DAG.get_or_init(|| Mutex::new(Dag::new()))
}
