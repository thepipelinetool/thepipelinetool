use std::cmp::max;

use chrono::Utc;
use clap::{command, Command, arg, value_parser};
use runner::{local::LocalRunner, Runner, DefRunner};
use saffron::{parse::{CronExpr, English}, Cron};
use utils::execute_function;

use crate::dag::DAG;

impl<'a> DAG<'a> {

    pub fn parse_cli(&self) {
        let command = command!()
            .about(format!("DAG CLI Tool"))
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
                        Command::new("local").about("Runs dag locally").arg(
                            arg!(
                                [mode] "Mode for running locally"
                            )
                            .required(false)
                            .value_parser(value_parser!(String))
                            .default_values(["max", "--blocking"])
                            .default_missing_value("max"),
                        ),
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
                    println!("{}", serde_json::to_string_pretty(&self.options).unwrap());
                }
                "describe" => {
                    println!("Task count: {}", self.nodes.len());
                    println!(
                        "Functions: {:#?}",
                        self.functions.keys().collect::<Vec<&String>>()
                    );

                    if let Some(schedule) = self.options.schedule {
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

                                if let Some(start_date) = self.options.start_date {
                                    println!("Start date: {start_date}");
                                } else {
                                    println!("Start date: None");
                                }

                                println!("Upcoming:");
                                let futures = cron.clone().iter_from(
                                    if let Some(start_date) = self.options.start_date {
                                        if self.options.catchup {
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
                                    if let Some(end_date) = self.options.end_date {
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
                    println!("{}", serde_json::to_string_pretty(&self.nodes).unwrap());
                }
                "edges" => {
                    println!("{}", serde_json::to_string_pretty(&self.edges).unwrap());
                }
                "graph" => {
                    print!("{}", self.get_initial_mermaid_graph());
                }
                "hash" => {
                    print!("{}", self.hash());
                }
                "tree" => {
                    let mut runner = LocalRunner::new("", &self.nodes, &self.edges);
                    let dag_run_id = runner.enqueue_run("local", "");
                    let tasks = runner
                        .get_default_tasks()
                        .iter()
                        .filter(|t| runner.get_upstream(&dag_run_id, &t.id).is_empty())
                        .map(|t| t.id)
                        .collect::<Vec<usize>>();

                    let mut output = format!("DAG\n");
                    let mut ts: Vec<usize> = vec![];

                    for (index, child) in tasks.iter().enumerate() {
                        let is_last = index == tasks.len() - 1;

                        let connector = if is_last { "└── " } else { "├── " };
                        ts.push(*child);
                        output.push_str(&runner.get_tree(
                            &dag_run_id,
                            child,
                            1,
                            connector,
                            vec![is_last],
                            &mut ts,
                        ));
                    }
                    println!("{}", output);
                    println!("{:?}", ts);
                }
                "run" => {
                    let matches = matches.subcommand_matches("run").unwrap();
                    if let Some(subcommand) = matches.subcommand_name() {
                        match subcommand {
                            "local" => {
                                let sub_matches = matches.subcommand_matches("local").unwrap();
                                let mode = sub_matches.get_one::<String>("mode").unwrap();

                                let max_threads = max(
                                    usize::from(std::thread::available_parallelism().unwrap()) - 1,
                                    1,
                                );
                                let thread_count = match mode.as_str() {
                                    "--blocking" => 1,
                                    "max" => max_threads,
                                    _ => mode.parse::<usize>().unwrap(),
                                };
                                LocalRunner::new("", &self.nodes, &self.edges)
                                    .run_dag_local(thread_count);
                            }
                            "function" => {
                                let sub_matches = matches.subcommand_matches("function").unwrap();
                                let function_name =
                                    sub_matches.get_one::<String>("function_name").unwrap();
                                let in_path = sub_matches.get_one::<String>("in_path").unwrap();
                                let out_path = sub_matches.get_one::<String>("out_path").unwrap();

                                if self.functions.contains_key(function_name) {
                                    execute_function(
                                        in_path,
                                        out_path,
                                        &self.functions[function_name],
                                    );
                                } else {
                                    panic!(
                                        "no such function {function_name}\navailable functions: {:#?}",
                                        self.functions.keys().collect::<Vec<&String>>()
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
}