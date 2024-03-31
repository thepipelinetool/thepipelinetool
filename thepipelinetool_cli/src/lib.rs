use std::{cmp::max, collections::HashSet, env, path::Path, process};

use chrono::Utc;
use clap::{arg, command, value_parser, ArgMatches, Command as CliCommand};
use thepipelinetool::dev::*;
use thepipelinetool_runner::{
    blanket::BlanketRunner,
    in_memory::{run_in_memory, InMemoryRunner},
    options::DagOptions,
    Runner,
};

pub mod yaml;

pub fn create_commands() -> CliCommand {
    command!()
        .about("DAG CLI Tool")
        // .subcommand(CliCommand::new("describe").about("Describes the DAG"))
        .subcommand(
            CliCommand::new("describe")
                .about("Run complete DAG or function by name")
                .arg_required_else_help(true)
                .subcommand(CliCommand::new("tasks").about("Displays tasks as JSON"))
                .subcommand(CliCommand::new("edges").about("Displays edges as JSON"))
                .subcommand(CliCommand::new("hash").about("Displays hash as JSON"))
                .subcommand(CliCommand::new("options").about("Displays options as JSON")),
        )
        .subcommand(CliCommand::new("check").about("Check for circular depencencies"))
        .subcommand(
            CliCommand::new("graph")
                .about("Displays graph")
                .arg_required_else_help(true)
                .arg(
                    arg!(
                        [graph_type] "Type of graph to output"
                    )
                    .required(true)
                    .value_parser(value_parser!(String))
                    .default_values(["mermaid", "graphite"])
                    .default_missing_value("mermaid"),
                ),
        )
        .subcommand(CliCommand::new("tree").about("Displays tree"))
        .subcommand(
            CliCommand::new("run")
                .about("Run complete DAG or function by name")
                .arg_required_else_help(true)
                .subcommand(
                    CliCommand::new("in_memory")
                        .about("Runs this DAG in memory")
                        .arg(
                            arg!(
                                [num_threads] "Max number of threads for parallel execution"
                            )
                            .required(false)
                            .value_parser(value_parser!(String))
                            .default_value("max")
                            .default_missing_value("max"),
                        ),
                )
                .subcommand(
                    CliCommand::new("function")
                        .about("Runs function")
                        .arg(
                            arg!(
                                <function_name> "Function name"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <in_path> "Input file"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <out_path> "Output file"
                            )
                            .required(false),
                        ),
                )
                .subcommand_required(true),
        )
        .subcommand_required(true)
}

pub fn process_subcommands(
    dag_path: &Path,
    dag_name: &str,
    subcommand_name: &str,
    tasks: &[Task],
    edges: &HashSet<(usize, usize)>,
    options: &DagOptions,
    matches: &ArgMatches,
) {
    match subcommand_name {
        "describe" => {
            let matches = matches.subcommand_matches("describe").unwrap();
            if let Some(subcommand) = matches.subcommand_name() {
                match subcommand {
                    "tasks" => display_tasks(),
                    "edges" => display_edges(),
                    "hash" => display_hash(tasks, edges),
                    "options" => display_options(options),
                    _ => {}
                }
            }
        }
        // "describe" => describe(tasks),
        // "tasks" => display_tasks(),
        // "edges" => display_edges(),
        "graph" => {
            let matches = matches.subcommand_matches("graph").unwrap();
            if let Some(subcommand) = matches.get_one::<String>("graph_type") {
                match subcommand.as_str() {
                    "mermaid" => display_mermaid_graph(tasks, edges),
                    "graphite" => display_graphite_graph(tasks, edges),
                    o => panic!("undefined graph type: {o}"),
                }
            }
        }
        "tree" => display_tree(tasks, edges, dag_path),
        "check" => check_circular_dependencies(tasks, edges),
        "run" => {
            let matches = matches.subcommand_matches("run").unwrap();
            if let Some(subcommand) = matches.subcommand_name() {
                // dbg!(subcommand);
                match subcommand {
                    "in_memory" => {
                        let num_threads = match matches
                            .subcommand_matches("in_memory")
                            .unwrap()
                            .get_one::<String>("num_threads")
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
                        let success = run_in_memory(
                            tasks,
                            edges,
                            dag_name.to_string(),
                            env::args().next().unwrap(),
                            num_threads,
                        );

                        process::exit(success);
                    }
                    "function" => run_function(matches),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn describe(tasks: &[Task]) {
    // TODO

    println!("Task count: {}", tasks.len());
}

fn display_options(options: &DagOptions) {
    println!("{}", serde_json::to_string_pretty(options).unwrap());
}

fn display_hash(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let hash = hash_dag(
        &serde_json::to_string(tasks).unwrap(),
        &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
    );
    print!("{hash}");
}

fn display_mermaid_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let mut runner = InMemoryRunner::new(tasks, edges);
    runner.enqueue_run("in_memory", "", Utc::now());

    let graph = runner.get_mermaid_graph(0);
    print!("{graph}");
}

fn display_graphite_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let mut runner = InMemoryRunner::new(tasks, edges);
    runner.enqueue_run("in_memory", "", Utc::now());

    let graph = runner.get_graphite_graph(0);
    print!("{}", serde_json::to_string_pretty(&graph).unwrap());
}

fn check_circular_dependencies(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
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

fn display_tree(tasks: &[Task], edges: &HashSet<(usize, usize)>, dag_path: &Path) {
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
        output.push_str(&runner.get_tree(
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

pub fn load_from_binary(dag_name: &str) {
    let tasks_from_json: Vec<Task> = serde_json::from_str(
        run_bash_commmand(&vec![dag_name, "describe", "tasks"], true)
            .as_str()
            .unwrap(),
    )
    .unwrap();

    for task in tasks_from_json {
        get_tasks().write().unwrap().insert(task.id, task);
    }

    let edges_from_json: Vec<(usize, usize)> = serde_json::from_str(
        run_bash_commmand(&vec![dag_name, "describe", "edges"], true)
            .as_str()
            .unwrap(),
    )
    .unwrap();

    for edge in edges_from_json {
        get_edges().write().unwrap().insert(edge);
    }
}
