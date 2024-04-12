use std::{env, path::Path, process};

use check_for_cycles::check_for_cycles;
use chrono::Utc;
use clap::ArgMatches;
use display_hash::display_hash;
use display_tree::display_tree;
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::{
    backend::Run, blanket_backend::BlanketBackend, in_memory_backend::InMemoryBackend,
    pipeline_options::PipelineOptions,
};

use std::collections::HashSet;

use crate::in_memory_runner::run_in_memory;

pub mod check_for_cycles;
pub mod commands;
pub mod display_hash;
pub mod display_tree;
mod in_memory_runner;
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

fn display_options(options: Option<PipelineOptions>) {
    if let Some(options) = options {
        print!("{}", serde_json::to_string_pretty(&options).unwrap());
    } else {
        print!("no options");
    }
}

pub fn process_subcommands(
    // pipeline_source: Option<String>,
    // pipeline_name: &str,
    subcommand_name: &str,
    options: Option<PipelineOptions>,
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

        "tree" => display_tree(tasks, edges),
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

                    let trigger_params = match matches
                        .subcommand_matches("in_memory")
                        .unwrap()
                        .get_one::<String>("params")
                        .unwrap()
                        .as_str()
                    {
                        "" => None,
                        any => Some(serde_json::from_str(any).expect("error parsing params")),
                    };

                    check_for_cycles(tasks, edges);

                    let mut backend = InMemoryBackend::new(tasks, edges);
                    let run = Run::dummy();
                    backend.enqueue_run(&run, trigger_params).unwrap();

                    run_in_memory(
                        &mut backend,
                        max_parallelism,
                        // pipeline_source,
                        env::args().next().unwrap(),
                    );

                    let exit_code = backend.get_run_status(run.run_id).unwrap();
                    // dbg!(backend.temp_queue);

                    process::exit(exit_code);
                }
                "function" => run_function(matches),
                _ => {}
            }
        }
        _ => {}
    }
}
