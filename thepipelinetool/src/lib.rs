use std::{env, path::Path, process};

use check_for_cycles::check_for_cycles;
use chrono::Utc;
use clap::ArgMatches;
use display_hash::display_hash;
use display_tree::display_tree;
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::{
    blanket_backend::BlanketBackend, in_memory_backend::InMemoryBackend,
    in_memory_runner::InMemoryRunner, options::DagOptions, run,
};

use std::collections::HashSet;

use thepipelinetool_core::dev::{get_default_graphite_graph, get_default_mermaid_graph, Task};

pub mod check_for_cycles;
pub mod commands;
pub mod display_hash;
pub mod display_tree;
pub mod read_from_executable;
pub mod read_from_yaml;
pub mod templating;

pub fn display_default_mermaid_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    print!("{}", get_default_mermaid_graph(tasks, edges));
}

pub fn display_default_graphite_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    print!(
        "{}",
        serde_json::to_string_pretty(&get_default_graphite_graph(tasks, edges)).unwrap()
    );
}

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
            "mermaid" => display_default_mermaid_graph(tasks, edges),
            "graphite" => display_default_graphite_graph(tasks, edges),
            _ => {}
        },

        "tree" => display_tree(tasks, edges, dag_path),
        "check" => check_for_cycles(tasks, edges),
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
                        "max" => get_default_max_parallelism(),
                        any => any.parse::<usize>().unwrap(),
                    };
                    assert!(max_parallelism > 0);

                    check_for_cycles(tasks, edges);
                    // let mut backend = runner.backend;
                    let mut runner = InMemoryRunner {
                        backend: Box::new(InMemoryBackend::new(tasks, edges)),
                        tpt_path: env::args().next().unwrap(),
                        dag_path: dag_path.to_path_buf(),
                    };
                    let run_id = runner.backend.enqueue_run("", "", Utc::now());

                    run(&mut runner, max_parallelism);

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
