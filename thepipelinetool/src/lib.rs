use std::{cmp::max, env, path::Path, process, thread::{self, JoinHandle}};

use chrono::Utc;
use circular_dependencies::check_circular_dependencies;
use clap::ArgMatches;
use graph::{display_graphite_graph, display_mermaid_graph};
use hash::display_hash;
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::{
    blanket_backend::BlanketBackend,
    in_memory::{InMemoryBackend, InMemoryRunner},
    options::DagOptions,
    run,
};
use tree::display_tree;

pub mod circular_dependencies;
pub mod commands;
pub mod executable;
pub mod graph;
pub mod hash;
pub mod template;
pub mod tree;
pub mod yaml;

fn display_options(options: &DagOptions) {
    print!("{}", serde_json::to_string_pretty(options).unwrap());
}

pub fn process_subcommands(
    dag_path: &Path,
    dag_name: &str,
    subcommand_name: &str,
    options: &DagOptions,
    matches: &ArgMatches,
) {
    let tasks = &get_tasks().read().unwrap();
    let edges = &get_edges().read().unwrap();

    match subcommand_name {
        "describe" => match matches
            .subcommand_matches("describe")
            .unwrap()
            .subcommand_name()
            .unwrap()
        {
            "tasks" => display_tasks(),
            "edges" => display_edges(),
            "hash" => display_hash(tasks, edges),
            "options" => display_options(options),
            _ => {}
        },
        "graph" => match matches
            .subcommand_matches("graph")
            .unwrap()
            .get_one::<String>("graph_type")
            .unwrap()
            .as_str()
        {
            "mermaid" => display_mermaid_graph(tasks, edges),
            "graphite" => display_graphite_graph(tasks, edges),
            _ => {}
        },

        "tree" => display_tree(tasks, edges, dag_path),
        "check" => check_circular_dependencies(tasks, edges),
        "run" => {
            let matches = matches.subcommand_matches("run").unwrap();
            match matches.subcommand_name().unwrap() {
                "in_memory" => {
                    let max_parallelism = match matches
                        .subcommand_matches("in_memory")
                        .unwrap()
                        .get_one::<String>("max_parallelism")
                        .unwrap()
                        .as_str()
                    {
                        "max" => max(
                            usize::from(std::thread::available_parallelism().unwrap()) - 1,
                            1,
                        ),
                        any => any.parse::<usize>().unwrap(),
                    };

                    check_circular_dependencies(tasks, edges);
                    // let mut backend = runner.backend;
                    let mut runner = InMemoryRunner {
                        backend: Box::new(InMemoryBackend::new(tasks, edges)),
                        tpt_path: env::args().next().unwrap(),
                        max_parallelism,
                        dag_path: dag_path.to_path_buf(),
                    };
                    let run_id = runner.backend.enqueue_run("", "", Utc::now());

                    run(&mut runner);

                    let exit_code = runner.backend.get_run_status(run_id);

                    process::exit(exit_code);
                }
                "function" => run_function(matches),
                _ => {}
            }
        }
        _ => {}
    }
}