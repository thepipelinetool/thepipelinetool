use thepipelinetool::dev::{get_edges, get_tasks, Task};
use thepipelinetool_utils::run_bash_commmand;

pub fn load_from_binary(dag_name: &str) {
    let tasks_from_json: Vec<Task> = serde_json::from_value(run_bash_commmand(
        &[dag_name, "describe", "tasks"],
        true,
        true,
    ))
    .unwrap();

    for task in tasks_from_json {
        get_tasks().write().unwrap().insert(task.id, task);
    }

    let edges_from_json: Vec<(usize, usize)> = serde_json::from_value(run_bash_commmand(
        &[dag_name, "describe", "edges"],
        true,
        true,
    ))
    .unwrap();

    for edge in edges_from_json {
        get_edges().write().unwrap().insert(edge);
    }
}
