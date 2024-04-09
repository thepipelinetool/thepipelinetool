use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::Command,
    sync::{Arc, OnceLock},
};

use parking_lot::Mutex;
use thepipelinetool_core::dev::Task;
use thepipelinetool_runner::{get_dag_path_by_name, get_dags_dir, options::DagOptions};

use crate::env::get_tpt_command;

type StaticServerTasks = Arc<Mutex<HashMap<String, Vec<Task>>>>;
type StaticServerHashes = Arc<Mutex<HashMap<String, String>>>;
type StaticServerEdges = Arc<Mutex<HashMap<String, HashSet<(usize, usize)>>>>;
type StaticServerDagOptions = Arc<Mutex<HashMap<String, DagOptions>>>;

static TASKS: OnceLock<StaticServerTasks> = OnceLock::new();
static HASHES: OnceLock<StaticServerHashes> = OnceLock::new();
static EDGES: OnceLock<StaticServerEdges> = OnceLock::new();
static DAG_OPTIONS: OnceLock<StaticServerDagOptions> = OnceLock::new();

pub fn _get_default_tasks(dag_name: &str) -> Option<Vec<Task>> {
    let mut tasks = TASKS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock();

    if !tasks.contains_key(dag_name) {
        let dag_path = get_dag_path_by_name(dag_name);
        if dag_path.is_none() {
            return None;
        }

        let output = Command::new(get_tpt_command())
            .arg(dag_path.unwrap())
            .arg("describe")
            .arg("tasks")
            .output()
            .expect("failed to run");

        tasks.insert(
            dag_name.to_owned(),
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap(),
        );
    }

    Some(tasks.get(dag_name).unwrap().clone())
}

pub fn _get_hash(dag_name: &str) -> String {
    let mut hashes = HASHES
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock();

    if !hashes.contains_key(dag_name) {
        let dags_dir = &get_dags_dir();
        let path: PathBuf = [dags_dir, dag_name].iter().collect();
        let output = Command::new(get_tpt_command())
            .arg(path)
            .arg("describe")
            .arg("hash")
            .output()
            .expect("failed to run");

        hashes.insert(
            dag_name.to_owned(),
            String::from_utf8_lossy(&output.stdout).to_string(),
        );
    }

    hashes.get(dag_name).unwrap().to_string()
}

pub fn _get_default_edges(dag_name: &str) -> Option<HashSet<(usize, usize)>> {
    let mut edges = EDGES
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock();

    if !edges.contains_key(dag_name) {
        let dag_path = get_dag_path_by_name(dag_name);
        if dag_path.is_none() {
            return None;
        }
        let output = Command::new(get_tpt_command())
            .arg(dag_path.unwrap())
            .arg("describe")
            .arg("edges")
            .output()
            .expect("failed to run");

        edges.insert(
            dag_name.to_owned(),
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap(),
        );
    }

    Some(edges.get(dag_name).unwrap().clone())
}

pub fn _get_options(dag_name: &str) -> Option<DagOptions> {
    let mut dag_options = DAG_OPTIONS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock();

    if !dag_options.contains_key(dag_name) {
        let dag_path = get_dag_path_by_name(dag_name);
        if dag_path.is_none() {
            return None;
        }

        let output = Command::new(get_tpt_command())
            .arg(dag_path.unwrap())
            .arg("describe")
            .arg("options")
            .output()
            .expect("failed to run. is tpt installed?");

        dag_options.insert(
            dag_name.to_owned(),
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap(),
        );
        // TODO verify schedule string
    }

    Some(dag_options.get(dag_name).unwrap().clone())
}

//
// pub fn _set_options(dag_name: &str, options: DagOptions) {
//     let mut path = _get_dag_path_by_name(dag_name);
//     path.set_extension("json");
//     value_to_file(&options, &path);

//     let mut dag_options = DAG_OPTIONS
//         .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
//         .lock();

//     dag_options.remove(dag_name);
// }
