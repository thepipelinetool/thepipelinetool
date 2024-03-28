use std::{
    cmp::max,
    collections::HashSet,
    env,
    path::Path,
    process::{self, Command},
};

use chrono::Utc;
use clap::{arg, command, value_parser, Arg, ArgMatches, Command as CliCommand};
use serde_json::json;
use thepipelinetool::dev::*;
use thepipelinetool_reader::yaml::read_from_yaml;
use thepipelinetool_runner::{blanket::BlanketRunner, in_memory::{run_in_memory, InMemoryRunner}, Runner};

fn create_commands() -> CliCommand {
    command!()
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
        .subcommand_required(true)
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("dag"));
    let matches = command.get_matches();
    let dag_name = matches.get_one::<String>("dag").expect("required");
    let dag_path = Path::new(dag_name);

    let subcommand_name = matches.subcommand_name().unwrap();

    match get_dag_type_by_path(dag_path.to_path_buf()) {
        DagType::Binary => {
            if args[2..4] == ["run", "function"] {
                let mut cmd = Command::new(dag_path);
                cmd.args(&mut args[2..]);
                let (exit_status, _) = spawn(
                    cmd,
                    Box::new(|x| print!("{x}")),
                    Box::new(|x| eprint!("{x}")),
                );
                process::exit(exit_status.code().unwrap());
            }

            let (load_tasks, load_edges) = match subcommand_name {
                "tasks" => (true, false),
                "edges" => (false, true),
                _ => (true, true),
            };

            if load_tasks {
                let tasks_from_json: Vec<Task> =
                    serde_json::from_str(run_command(json!([dag_name, "tasks"])).as_str().unwrap())
                        .unwrap();

                for t in tasks_from_json {
                    get_tasks().write().unwrap().insert(t.id, t);
                }
            }

            if load_edges {
                let edges_from_json: Vec<(usize, usize)> =
                    serde_json::from_str(run_command(json!([dag_name, "edges"])).as_str().unwrap())
                        .unwrap();

                for e in edges_from_json {
                    get_edges().write().unwrap().insert(e);
                }
            }
        }
        DagType::YAML => {
            for template_args in read_from_yaml(dag_path) {
                add_task(run_command, template_args, &TaskOptions::default());
            }
        }
    }

    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();
    process_subcommands(&dag_name, subcommand_name, &tasks, &edges, &matches);
}

fn process_subcommands(
    dag_path: &str,
    subcommand_name: &str,
    tasks: &[Task],
    edges: &HashSet<(usize, usize)>,
    matches: &ArgMatches,
) {
    match subcommand_name {
        "describe" => describe(tasks),
        "tasks" => display_tasks(),
        "edges" => display_edges(),
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
        "hash" => display_hash(tasks, edges),
        "tree" => display_tree(tasks, edges),
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

                        run_in_memory(&tasks, &edges, dag_path.to_string(), num_threads);
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

fn display_hash(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let hash = hash_dag(
        &serde_json::to_string(&*tasks).unwrap(),
        &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
    );
    print!("{hash}");
}

fn display_mermaid_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let mut runner = InMemoryRunner::new(&tasks, &edges);
    runner.enqueue_run("in_memory", "", Utc::now());

    let graph = runner.get_mermaid_graph(0);
    print!("{graph}");
}

fn display_graphite_graph(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let mut runner = InMemoryRunner::new(&tasks, &edges);
    runner.enqueue_run("in_memory", "", Utc::now());

    let graph = runner.get_graphite_graph(0);
    print!("{}", serde_json::to_string_pretty(&graph).unwrap());
}

fn display_tree(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
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
}
