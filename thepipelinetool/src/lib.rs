use std::{env, path::Path, process};

use check_for_cycles::check_for_cycles;
use chrono::Utc;
use clap::ArgMatches;
use display_hash::display_hash;
use display_tree::display_tree;
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::{
    backend::Run, blanket_backend::BlanketBackend, in_memory_backend::InMemoryBackend,
    pipeline::Pipeline, pipeline_options::PipelineOptions,
};

use anyhow::Result;
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

fn display_options(options: &PipelineOptions) {
    print!("{}", serde_json::to_string_pretty(options).unwrap());
}

pub fn process_subcommands(
    pipeline_path: &Path,
    pipeline_name: &str,
    subcommand_name: &str,
    options: &PipelineOptions,
    matches: &ArgMatches,
) -> Result<()> {
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

        "tree" => display_tree(tasks, edges, pipeline_path),
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

                    let mut backend = InMemoryBackend::new(pipeline_path.to_str().unwrap(), tasks, edges);
                    let run = Run::dummy();
                    backend.enqueue_run(&run, trigger_params).unwrap();

                    run_in_memory(
                        &mut backend,
                        max_parallelism,
                        // pipeline_path.to_path_buf().to_str().unwrap().to_string(),
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
        "upload" => {
            let endpoint = matches
                .subcommand_matches("upload")
                .unwrap()
                .get_one::<String>("endpoint")
                .expect("required");
            // dbg!(endpoint);

            let pipeline = Pipeline {
                name: pipeline_name.to_string(),
                path: pipeline_path.to_str().expect("").to_string(),
                options: options.clone(),
                tasks: get_tasks().read().unwrap().to_vec(),
                edges: get_edges().read().unwrap().to_owned(),
            };

            let client = reqwest::blocking::Client::new();
            let res = client
                .post(endpoint)
                .json(&pipeline)
                .send()?;
            assert!(res.status().is_success());

            // dbg!(pipeline);
        }
        _ => {}
    };
    Ok(())
}
