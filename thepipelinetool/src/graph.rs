use std::collections::{HashMap, HashSet};

use thepipelinetool_core::dev::{get_graphite_graph, get_mermaid_graph, Task, TaskStatus};

pub fn display_default_mermaid_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let task_statuses: Vec<(String, TaskStatus)> = tasks
        .iter()
        .map(|t| (t.name.clone(), TaskStatus::Pending))
        .collect();

    let mut upstream_ids: HashMap<usize, Vec<usize>> =
        HashMap::from_iter(tasks.iter().map(|t| (t.id, vec![])));
    for (upstream_id, downstream_id) in edges {
        upstream_ids
            .get_mut(downstream_id)
            .unwrap()
            .push(*upstream_id);
    }

    print!("{}", get_mermaid_graph(&task_statuses, &upstream_ids));
}

pub fn display_default_graphite_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let task_statuses: Vec<(usize, String, TaskStatus)> = tasks
        .iter()
        .map(|task| (task.id, task.name.clone(), TaskStatus::Pending))
        .collect();

    let mut downstream_ids: HashMap<usize, Vec<usize>> =
        HashMap::from_iter(tasks.iter().map(|t| (t.id, vec![])));
    for (upstream_id, downstream_id) in edges {
        downstream_ids
            .get_mut(upstream_id)
            .unwrap()
            .push(*downstream_id);
    }

    print!(
        "{}",
        serde_json::to_string_pretty(&get_graphite_graph(&task_statuses, &downstream_ids)).unwrap()
    );
}
