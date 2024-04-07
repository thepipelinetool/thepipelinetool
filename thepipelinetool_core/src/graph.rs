use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};
use thepipelinetool_task::{task_status::TaskStatus, Task};

pub fn get_mermaid_graph(
    task_statuses: &[(String, TaskStatus)],
    upstream_ids: &HashMap<usize, Vec<usize>>,
) -> String {
    // let task_statuses: Vec<(String, TaskStatus)> = self
    //     .get_all_tasks(dag_run_id)
    //     .iter()
    //     .map(|t| (t.name.clone(), self.get_task_status(dag_run_id, t.id)))
    //     .collect();

    let mut out = "".to_string();
    out += "flowchart TD\n";

    for (task_id, (task_name, task_status)) in task_statuses.iter().enumerate() {
        let styling = get_styling_for_status(task_status);
        out += &format!("  id{task_id}({task_name}_{task_id})\n");
        out += &format!("  style id{task_id} {styling}\n");

        for edge_id in &upstream_ids[&task_id] {
            out += &format!("  id{edge_id}-->id{task_id}\n");
        }
    }

    out
}

pub fn get_graphite_graph(
    task_statuses: &[(usize, String, TaskStatus)],
    downstream_ids: &HashMap<usize, Vec<usize>>,
) -> Vec<Value> {
    task_statuses
        .iter()
        .map(|(task_id, task_name, task_status)| {
            let name = format!("{task_name}_{task_id}");
            let next = downstream_ids[task_id]
                .iter()
                .map(|downstream_id| json!({"outcome": downstream_id.to_string()}))
                .collect::<Vec<Value>>();
            json!({
                "id": task_id.to_string(),
                "name": name,
                "next": next,
                "status": serde_json::to_string(task_status).unwrap(),
            })
        })
        .collect()
}

fn get_styling_for_status(task_status: &TaskStatus) -> String {
    match task_status {
        TaskStatus::Pending => "color:black,stroke:grey,fill:white,stroke-width:4px".into(),
        TaskStatus::Success => "color:black,stroke:green,fill:white,stroke-width:4px".into(),
        TaskStatus::Failure => "color:black,stroke:red,fill:white,stroke-width:4px".into(),
        TaskStatus::Running => "color:black,stroke:#90EE90,fill:white,stroke-width:4px".into(),
        TaskStatus::RetryPending => "color:black,stroke:orange,fill:white,stroke-width:4px".into(),
        TaskStatus::Skipped => "color:black,stroke:pink,fill:white,stroke-width:4px".into(),
    }
}

pub fn get_default_graphite_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) -> Vec<Value> {
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
    get_graphite_graph(&task_statuses, &downstream_ids)
}

pub fn get_default_mermaid_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) -> String {
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

    get_mermaid_graph(&task_statuses, &upstream_ids)
}
