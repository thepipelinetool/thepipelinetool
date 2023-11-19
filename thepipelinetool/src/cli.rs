use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    env,
    hash::{Hash, Hasher},
};

use chrono::Utc;
use clap::{arg, command, Command};
// use graph::dag::get_dag;
use runner::{blanket::BlanketRunner, in_memory::InMemoryRunner, Runner};
use saffron::{
    parse::{CronExpr, English},
    Cron,
};
use utils::{execute_function, to_base62};

use crate::{get_edges, get_functions, get_options, get_tasks};

// use crate::{options::DagOptions};

// use crate::hash;

// use crate::dag::Dag;

// impl Dag {

pub fn parse_cli() {
    // let tasks = get_tasks().lock().unwrap();
    // let edges = get_edges().lock().unwrap();
    // let options = get_options().lock().unwrap();
    // let functions = get_functions().lock().unwrap();

    let command = command!()
        .about("DAG CLI Tool")
        .subcommand(Command::new("describe").about("Describes the DAG"))
        .subcommand(Command::new("options").about("Displays options as JSON"))
        .subcommand(Command::new("tasks").about("Displays tasks as JSON"))
        .subcommand(Command::new("edges").about("Displays edges as JSON"))
        .subcommand(Command::new("hash").about("Displays hash as JSON"))
        .subcommand(Command::new("graph").about("Displays graph"))
        .subcommand(Command::new("tree").about("Displays tree"))
        .subcommand(
            Command::new("run")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("in_memory").about("Runs dag locally"), // .arg(
                                                                         //     arg!(
                                                                         //         [mode] "Mode for running locally"
                                                                         //     )
                                                                         //     .required(false)
                                                                         //     .value_parser(value_parser!(String))
                                                                         //     .default_values(["max", "--blocking"])
                                                                         //     .default_missing_value("max"),
                                                                         // ),
                )
                .subcommand(
                    Command::new("function")
                        .about("Runs function")
                        .arg(
                            arg!(
                                <function_name> "Function name"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <out_path> "Output file"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <in_path> "Input file"
                            )
                            .required(true),
                        ),
                )
                .subcommand_required(true),
        )
        .subcommand_required(true);

    let matches = command.get_matches();

    if let Some(subcommand) = matches.subcommand_name() {
        match subcommand {
            "options" => {
                let options = get_options().read().unwrap();

                println!(
                    "{}",
                    serde_json::to_string_pretty(&options.clone()).unwrap()
                );
            }
            "describe" => {
                let tasks = get_tasks().read().unwrap();
                let options = get_options().read().unwrap();
                let functions = get_functions().read().unwrap();

                println!("Task count: {}", tasks.len());
                println!(
                    "Functions: {:#?}",
                    functions.keys().collect::<Vec<&String>>()
                );

                if let Some(schedule) = &options.schedule {
                    println!("Schedule: {schedule}");
                    match schedule.parse::<CronExpr>() {
                        Ok(cron) => {
                            println!("Description: {}", cron.describe(English::default()));
                        }
                        Err(err) => {
                            println!("{err}: {schedule}");
                            return;
                        }
                    }

                    match schedule.parse::<Cron>() {
                        Ok(cron) => {
                            if !cron.any() {
                                println!("Cron will never match any given time!");
                                return;
                            }

                            if let Some(start_date) = options.start_date {
                                println!("Start date: {start_date}");
                            } else {
                                println!("Start date: None");
                            }

                            println!("Upcoming:");
                            let futures = cron.clone().iter_from(
                                if let Some(start_date) = options.start_date {
                                    if options.catchup || start_date > Utc::now() {
                                        start_date.into()
                                    } else {
                                        Utc::now()
                                    }
                                } else {
                                    Utc::now()
                                },
                            );
                            for time in futures.take(10) {
                                if !cron.contains(time) {
                                    println!("Failed check! Cron does not contain {}.", time);
                                    break;
                                }
                                if let Some(end_date) = options.end_date {
                                    if time > end_date {
                                        break;
                                    }
                                }
                                println!("  {}", time.format("%F %R"));
                            }
                        }
                        Err(err) => println!("{err}: {schedule}"),
                    }
                } else {
                    println!("No schedule set");
                }
            }
            "tasks" => {
                let tasks = get_tasks().read().unwrap();

                println!("{}", serde_json::to_string_pretty(&*tasks).unwrap());
            }
            "edges" => {
                let edges = get_edges().read().unwrap();

                println!("{}", serde_json::to_string_pretty(&*edges).unwrap());
            }
            "graph" => {
                let tasks = get_tasks().read().unwrap();
                let edges = get_edges().read().unwrap();

                let mut runner = InMemoryRunner::new("", &tasks, &edges);
                runner.enqueue_run("in_memory", "", Utc::now());

                let graph = runner.get_graphite_graph(0);
                print!("{}", serde_json::to_string_pretty(&graph).unwrap());
            }
            "hash" => {
                let tasks = get_tasks().read().unwrap();
                let edges = get_edges().read().unwrap();

                let hash = hash_dag(
                    &serde_json::to_string(&*tasks).unwrap(),
                    &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
                );
                print!("{hash}");
            }
            "tree" => {
                let tasks = get_tasks().read().unwrap();
                let edges = get_edges().read().unwrap();

                let mut runner = InMemoryRunner::new("", &tasks, &edges);
                let run_id = runner.enqueue_run("in_memory", "", Utc::now());
                let tasks = runner
                    .get_default_tasks()
                    .iter()
                    .filter(|t| runner.get_upstream(run_id, t.id).is_empty())
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
                println!("{:?}", task_ids_in_order);
            }
            "run" => {
                let matches = matches.subcommand_matches("run").unwrap();
                if let Some(subcommand) = matches.subcommand_name() {
                    match subcommand {
                        "in_memory" => {
                            let tasks = get_tasks().read().unwrap();
                            let edges = get_edges().read().unwrap();
                            let mut runner = InMemoryRunner::new("", &tasks, &edges);
                            let run_id = runner.enqueue_run("", "", Utc::now());
                            let default_tasks = runner.get_default_tasks();

                            for task in &default_tasks {
                                let mut visited = HashSet::new();
                                let mut path = vec![];
                                let circular_dependencies = runner.get_circular_dependencies(
                                    run_id,
                                    task.id,
                                    &mut visited,
                                    &mut path,
                                );

                                if let Some(deps) = circular_dependencies {
                                    panic!("{:?}", deps);
                                }
                            }

                            let current_executable_path = &env::args().next().unwrap();

                            while let Some(queued_task) = runner.pop_priority_queue() {
                                runner.work(run_id, queued_task, current_executable_path.as_str());
                            }
                        }
                        "function" => {
                            let functions = get_functions().read().unwrap();

                            let sub_matches = matches.subcommand_matches("function").unwrap();
                            let function_name =
                                sub_matches.get_one::<String>("function_name").unwrap();
                            let in_path = sub_matches.get_one::<String>("in_path").unwrap();
                            let out_path = sub_matches.get_one::<String>("out_path").unwrap();

                            if functions.contains_key(function_name) {
                                execute_function(in_path, out_path, &functions[function_name]);
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

fn hash_dag(nodes: &str, edges: &[(usize, usize)]) -> String {
    let mut hasher = DefaultHasher::new();
    let mut edges: Vec<&(usize, usize)> = edges.iter().collect();
    edges.sort();

    let to_hash = serde_json::to_string(&nodes).unwrap() + &serde_json::to_string(&edges).unwrap();
    to_hash.hash(&mut hasher);
    to_base62(hasher.finish())
}
