use thepipelinetool_core::dev::{get_edges, get_tasks, Task};
use thepipelinetool_utils::run_bash_command;

pub fn read_from_executable(pipeline_name: &str) {
    let tasks_from_json: Vec<Task> = serde_json::from_value(run_bash_command(
        &[pipeline_name, "describe", "tasks"],
        true,
        true,
    ))
    .unwrap();

    for task in tasks_from_json {
        get_tasks().write().unwrap().insert(task.id, task);
    }

    let edges_from_json: Vec<(usize, usize)> = serde_json::from_value(run_bash_command(
        &[pipeline_name, "describe", "edges"],
        true,
        true,
    ))
    .unwrap();

    for edge in edges_from_json {
        get_edges().write().unwrap().insert(edge);
    }
}
