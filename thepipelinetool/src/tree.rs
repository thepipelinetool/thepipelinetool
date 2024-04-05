use std::{collections::HashSet, path::Path};

use chrono::Utc;
use thepipelinetool_core::dev::Task;
use thepipelinetool_runner::{blanket::BlanketRunner, in_memory::InMemoryRunner, Runner};

pub fn display_tree(tasks: &[Task], edges: &HashSet<(usize, usize)>, dag_path: &Path) {
    let mut runner = InMemoryRunner::new(tasks, edges);
    let run_id = runner.enqueue_run("in_memory", "", Utc::now());
    let tasks = runner
        .get_default_tasks()
        .iter()
        .filter(|t| runner.get_task_depth(run_id, t.id) == 0)
        .map(|t| t.id)
        .collect::<Vec<usize>>();

    let mut output = format!("{}\n", dag_path.file_name().unwrap().to_str().unwrap());
    let mut task_ids_in_order: Vec<usize> = vec![];

    for (index, child) in tasks.iter().enumerate() {
        let is_last = index == tasks.len() - 1;

        let connector = if is_last { "└── " } else { "├── " };
        task_ids_in_order.push(*child);
        output.push_str(&get_tree(
            &runner,
            run_id,
            *child,
            1,
            connector,
            vec![is_last],
            &mut task_ids_in_order,
        ));
    }
    println!("{}", output);
}

fn get_tree(
    runner: &dyn Runner,
    run_id: usize,
    task_id: usize,
    _depth: usize,
    prefix: &str,
    prev_is_last: Vec<bool>,
    task_ids_in_order: &mut Vec<usize>,
) -> String {
    let children: Vec<usize> = runner.get_downstream(run_id, task_id);
    let mut output = format!(
        "{}{}_{}\n",
        prefix,
        runner.get_task_by_id(run_id, task_id).name,
        task_id,
    );

    for (index, child) in children.iter().enumerate() {
        let is_last = index == children.len() - 1;
        let child_prefix = prev_is_last.iter().fold(String::new(), |acc, &last| {
            if last {
                acc + "    "
            } else {
                acc + "│   "
            }
        });

        let connector = if is_last { "└── " } else { "├── " };
        let mut new_prev_is_last = prev_is_last.clone();
        new_prev_is_last.push(is_last);
        task_ids_in_order.push(*child);
        output.push_str(&get_tree(
            runner,
            run_id,
            *child,
            _depth + 1,
            &format!("{}{}", child_prefix, connector),
            new_prev_is_last,
            task_ids_in_order,
        ));
    }

    output
}
