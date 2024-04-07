use std::collections::HashSet;

use chrono::Utc;
use thepipelinetool_core::dev::Task;
use thepipelinetool_runner::{blanket_backend::BlanketBackend, in_memory::InMemoryBackend};

pub fn display_mermaid_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let mut runner = InMemoryBackend::new(tasks, edges);
    runner.enqueue_run("in_memory", "", Utc::now());

    let graph = runner.get_mermaid_graph(0);
    print!("{graph}");
}

pub fn display_graphite_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let mut runner = InMemoryBackend::new(tasks, edges);
    runner.enqueue_run("in_memory", "", Utc::now());

    let graph = runner.get_graphite_graph(0);
    print!("{}", serde_json::to_string_pretty(&graph).unwrap());
}
