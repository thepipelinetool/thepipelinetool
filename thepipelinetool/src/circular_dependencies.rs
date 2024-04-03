use std::{collections::HashSet, process};

use thepipelinetool_core::dev::Task;

pub fn check_circular_dependencies(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    if let Some(cycle_tasks) = get_circular_dependencies(tasks.len(), edges) {
        eprintln!(
            "cycle detected: {}",
            cycle_tasks
                .iter()
                .map(|i| format!("({}_{})", tasks[*i].name, i))
                .collect::<Vec<String>>()
                .join("-->")
        );
        process::exit(1);
    }
    println!("no circular dependencies found");
}

fn get_circular_dependencies(
    tasks_len: usize,
    edges: &HashSet<(usize, usize)>,
) -> Option<Vec<usize>> {
    for task in 0..tasks_len {
        let mut visited_tasks = HashSet::new();
        let mut cycle_tasks = vec![];

        if _get_circular_dependencies(
            edges,
            task,
            &mut visited_tasks,
            &mut cycle_tasks, //
        )
        .is_some()
        {
            return Some(cycle_tasks);
        }
    }
    None
}

fn get_upstream(id: usize, edges: &HashSet<(usize, usize)>) -> Vec<usize> {
    edges.iter().filter(|e| e.1 == id).map(|e| e.0).collect()
}

fn _get_circular_dependencies(
    edges: &HashSet<(usize, usize)>,
    current: usize,
    visited_tasks: &mut HashSet<usize>,
    cycle_tasks: &mut Vec<usize>,
) -> Option<Vec<usize>> {
    visited_tasks.insert(current);
    cycle_tasks.push(current);

    for neighbor in get_upstream(current, edges) {
        if !visited_tasks.contains(&neighbor) {
            return _get_circular_dependencies(edges, neighbor, visited_tasks, cycle_tasks);
        } else if cycle_tasks.contains(&neighbor) {
            cycle_tasks.push(neighbor);
            return Some(cycle_tasks.to_vec());
        }
    }

    cycle_tasks.pop();
    visited_tasks.remove(&current);
    None
}
