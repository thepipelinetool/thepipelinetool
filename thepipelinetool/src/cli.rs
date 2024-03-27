use std::collections::HashSet;
use std::path::Path;
use thepipelinetool_task::Task;
use thepipelinetool_utils::execute_function_using_json_str_args;

use std::{cmp::max, env, sync::mpsc::channel, thread};

use chrono::Utc;
use clap::{arg, command, value_parser, Command as CliCommand};

use thepipelinetool_runner::{blanket::BlanketRunner, in_memory::InMemoryRunner, Runner};
use thepipelinetool_utils::execute_function_using_json_files;

use crate::hash::hash_dag;
use crate::*;

fn describe() {
    let tasks: std::sync::RwLockReadGuard<'_, Vec<Task>> = get_tasks().read().unwrap();
    let functions = get_functions().read().unwrap();

    println!("Task count: {}", tasks.len());
    println!(
        "Functions: {:#?}",
        functions.keys().collect::<Vec<&String>>()
    );
}

fn display_tasks() {
    let tasks = get_tasks().read().unwrap();

    println!("{}", serde_json::to_string_pretty(&*tasks).unwrap());
}

fn display_edges() {
    let edges = get_edges().read().unwrap();

    println!("{}", serde_json::to_string_pretty(&*edges).unwrap());
}

fn display_hash() {
    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();

    let hash = hash_dag(
        &serde_json::to_string(&*tasks).unwrap(),
        &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
    );
    print!("{hash}");
}

fn display_graph(graph_type: &str) {
    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();

    let mut runner = InMemoryRunner::new(&tasks, &edges);
    runner.enqueue_run("in_memory", "", Utc::now());

    match graph_type {
        "mermaid" => {
            let graph = runner.get_mermaid_graph(0);
            print!("{graph}");
        }
        "graphite" => {
            let graph = runner.get_graphite_graph(0);
            print!("{}", serde_json::to_string_pretty(&graph).unwrap());
        }
        o => {
            panic!("undefined graph type: {o}");
        }
    }
}

fn display_tree() {
    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();

    let mut runner = InMemoryRunner::new(&tasks, &edges);
    let run_id = runner.enqueue_run("in_memory", "", Utc::now());
    let tasks = runner
        .get_default_tasks()
        .iter()
        .filter(|t| runner.get_task_depth(run_id, t.id) == 0)
        .map(|t| t.id)
        .collect::<Vec<usize>>();

    let mut output = "DAG\n".to_string();
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
    // println!("{:?}", task_ids_in_order);
}

pub fn run_in_memory(num_threads: usize) {
    dbg!(1);
    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();

    let mut runner = InMemoryRunner::new(&tasks.to_vec(), &edges);

    let run_id = runner.enqueue_run("", "", Utc::now());

    let default_tasks = runner.get_default_tasks();
    dbg!(2);
    // check for circular dependencies
    for task in &default_tasks {
        let mut visited = HashSet::new();
        let mut path = vec![];
        let circular_dependencies =
            runner.get_circular_dependencies(run_id, task.id, &mut visited, &mut path);

        if let Some(deps) = circular_dependencies {
            panic!("{:?}", deps);
        }
    }

    let (tx, rx) = channel();

    let mut thread_count = 0;

    for _ in 0..num_threads {
        let mut runner = runner.clone();
        let tx = tx.clone();
        dbg!(3);
        if let Some(queued_task) = runner.pop_priority_queue() {
            thread::spawn(move || {
                let current_executable_path = &env::args().next().unwrap();

                runner.work(run_id, &queued_task, current_executable_path.as_str());
                tx.send(()).unwrap();
            });

            thread_count += 1;
            if thread_count >= num_threads {
                break;
            }
        } else {
            break;
        }
    }

    for _ in rx.iter() {
        dbg!(4);
        thread_count -= 1;

        let mut runner = runner.clone();
        if let Some(queued_task) = runner.pop_priority_queue() {
            let tx = tx.clone();

            thread::spawn(move || {
                let current_executable_path = &env::args().next().unwrap();

                runner.work(run_id, &queued_task, current_executable_path.as_str());
                tx.send(()).unwrap();
            });

            thread_count += 1;

            if thread_count >= num_threads {
                continue;
            }
        }
        if thread_count == 0 {
            drop(tx);
            break;
        }
    }
    dbg!(5);
}

/// Parses command-line arguments and executes various tasks in the DAG CLI tool.
///
/// This function parses command-line arguments using the `command!` macro and executes
/// corresponding tasks based on the subcommands and options provided. It interacts with
/// the task management system to perform operations like displaying task information, running
/// tasks, and more.
///
/// The `parse_cli` function is typically called in the `main` function of your Rust application.
/// If you are using the #[dag] macro, it will automatically add a `parse_cli()` function call
/// to the end of the `main` function, simplifying the setup.
///
/// # Examples
///
///
/// ```rust
/// #[dag]
/// fn main() {
///     // your code here
///
///     // The #[dag] macro adds a parse_cli() function call to the end of the main function
/// }
/// ```
/// is equivalent to
/// ```rust
/// fn main() {
///     // your code here
///     parse_cli();
/// }
/// ```
///
/// The behavior of the CLI tool depends on the subcommands and options passed on the command
/// line. Use the "--help" command to see the CLI details.
pub fn parse_cli() {
    let command = command!()
        .about("DAG CLI Tool")
        .subcommand(CliCommand::new("describe").about("Describes the DAG"))
        // .subcommand(CliCommand::new("options").about("Displays options as JSON"))
        .subcommand(CliCommand::new("tasks").about("Displays tasks as JSON"))
        .subcommand(CliCommand::new("edges").about("Displays edges as JSON"))
        .subcommand(CliCommand::new("hash").about("Displays hash as JSON"))
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
        .subcommand_required(true);

    let matches = command.get_matches();

    if let Some(subcommand) = matches.subcommand_name() {
        match subcommand {
            "describe" => describe(),
            "tasks" => display_tasks(),
            "edges" => display_edges(),
            "graph" => {
                let matches = matches.subcommand_matches("graph").unwrap();
                if let Some(subcommand) = matches.get_one::<String>("graph_type") {
                    display_graph(subcommand.as_str());
                }
            }
            "hash" => display_hash(),
            "tree" => display_tree(),
            "run" => {
                let matches = matches.subcommand_matches("run").unwrap();
                if let Some(subcommand) = matches.subcommand_name() {
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
                            run_in_memory(num_threads);

                            // let tasks = runner
                            //     .get_default_tasks()
                            //     .iter()
                            //     .filter(|t| runner.get_task_depth(run_id, t.id) == 0)
                            //     .map(|t| t.id)
                            //     .collect::<Vec<usize>>();

                            // let mut output = "DAG\n".to_string();
                            // let mut task_ids_in_order: Vec<usize> = vec![];

                            // for (index, child) in tasks.iter().enumerate() {
                            //     let is_last = index == tasks.len() - 1;

                            //     let connector = if is_last { "└── " } else { "├── " };
                            //     task_ids_in_order.push(*child);
                            //     output.push_str(&runner.get_tree(
                            //         run_id,
                            //         *child,
                            //         1,
                            //         connector,
                            //         vec![is_last],
                            //         &mut task_ids_in_order,
                            //     ));
                            // }
                            // println!("{}", output);
                            // println!("{:?}", task_ids_in_order);
                        }
                        "function" => {
                            let functions = get_functions().read().unwrap();

                            let sub_matches = matches.subcommand_matches("function").unwrap();
                            let function_name =
                                sub_matches.get_one::<String>("function_name").unwrap();
                            let in_arg = sub_matches.get_one::<String>("in_path").unwrap();
                            let out_path_match = sub_matches.get_one::<String>("out_path");

                            if functions.contains_key(function_name) {
                                if let Some(out_path) = out_path_match {
                                    execute_function_using_json_files(
                                        Path::new(in_arg),
                                        Path::new(out_path),
                                        &functions[function_name],
                                    );
                                } else {
                                    execute_function_using_json_str_args(
                                        in_arg,
                                        &functions[function_name],
                                    );
                                }
                            } else {
                                panic!(
                                    "no such function {function_name}\navailable functions: {:#?}",
                                    functions.keys().collect::<Vec<&String>>()
                                )
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
