use std::{
    cmp::max,
    collections::HashSet,
    env,
    fs::File,
    path::Path,
    process::{self, Command},
    sync::mpsc::channel,
    thread,
};

use chrono::Utc;
use clap::{arg, command, value_parser, Arg, ArgMatches, Command as CliCommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thepipelinetool::{
    cli::*,
    get_edges, get_tasks,
    hash::hash_dag,
    operators::run_command,
    server::{add_task, BlanketRunner, InMemoryRunner, TaskOptions},
};
use thepipelinetool_runner::Runner;
use thepipelinetool_task::Task;
use thepipelinetool_utils::{get_dag_type_by_path, spawn, DagType};

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
    let args: Vec<String> = env::args().collect();
    // dbg!(&args);
    let command = create_commands().arg(Arg::new("dag"));
    let matches = command.get_matches();
    let dag_name = matches.get_one::<String>("dag").expect("required");
    let dag_path = Path::new(dag_name);
    match get_dag_type_by_path(dag_path.to_path_buf()) {
        DagType::Binary => {
            if let Some(subcommand) = matches.subcommand_name() {
                let _override: bool = match subcommand {
                    // "describe" => describe(),
                    "tasks" => true,
                    "edges" => true,
                    // "graph" => display_graph(&matches),
                    // "hash" => display_hash(),
                    // "tree" => display_tree(),
                    "run" => {
                        let matches = matches.subcommand_matches("run").unwrap();
                        // if let Some(subcommand) = matches.subcommand_name() {
                        match matches.subcommand_name() {
                            // "in_memory" => run_in_memory(matches),
                            Some("function") => true,
                            _ => false,
                        }
                        // }
                    }
                    _ => false,
                };

                // dbg!(_override);

                if _override {
                    // println!("{:#?}", args);
                    let mut cmd = Command::new(&args[1]);
                    cmd.args(&args[2..]);
                    let (exit_status, _) = spawn(
                        cmd,
                        Box::new(move |s| print!("{s}")),
                        Box::new(move |s| eprint!("{s}")),
                    );
                    process::exit(exit_status.code().unwrap());
                    // return;
                }

                let tasks_json: Vec<Task> =
                    serde_json::from_str(run_command(json!([args[1], "tasks"])).as_str().unwrap())
                        .unwrap();

                for t in tasks_json {
                    get_tasks().write().unwrap().insert(t.id, t);
                }

                let edges_json: Vec<(usize, usize)> =
                    serde_json::from_str(run_command(json!([args[1], "edges"])).as_str().unwrap())
                        .unwrap();

                for e in edges_json {
                    get_edges().write().unwrap().insert(e);
                }
            }
        }
        DagType::YAML => {
            // dbg!(dag_path);
            let value: Value = serde_yaml::from_reader(File::open(dag_path).unwrap()).unwrap();
            // load tasks and edges
            // dbg!(value);

            // let json_tasks = value.clone();
            if !value.as_object().unwrap().contains_key("tasks") {
                return;
            }

            let tasks = value["tasks"].as_object().unwrap();
            let task_templates: Vec<TaskTemplate> = tasks
                .iter()
                .map(|(key, value)| {
                    // dbg!(value);
                    let mut template: TaskTemplate = serde_json::from_value(value.clone()).unwrap();
                    template.function_name = key.to_string();
                    template
                })
                .collect();

            for task_template in &task_templates {
                let template_args = serde_json::to_value(&task_template.command).unwrap();
                add_task(run_command, template_args, &TaskOptions::default());
            }
            // dbg!(&task_templates);
        }
    }
    process_commands(&matches, &dag_name);
}

fn process_commands(matches: &ArgMatches, dag_path: &str) {
    if let Some(subcommand) = matches.subcommand_name() {
        match subcommand {
            "describe" => describe(),
            "tasks" => display_tasks(),
            "edges" => display_edges(),
            "graph" => display_graph(&matches),
            "hash" => display_hash(),
            "tree" => display_tree(),
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
                            run_in_memory(dag_path.to_string(), num_threads);
                        }
                        "function" => run_function(matches),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskTemplate {
    #[serde(default)]
    pub function_name: String,

    #[serde(default)]
    pub command: Vec<String>,

    // #[serde(default)]
    // pub args: Value,
    #[serde(default)]
    pub options: TaskOptions,

    #[serde(default)]
    pub lazy_expand: bool,

    #[serde(default)]
    pub is_dynamic: bool,

    #[serde(default)]
    pub is_branch: bool,
}

impl Default for TaskTemplate {
    fn default() -> Self {
        Self {
            function_name: "".to_string(),
            command: Vec::new(),
            // args: json!({}),
            options: Default::default(),
            lazy_expand: false,
            is_dynamic: false,
            is_branch: false,
        }
    }
}

pub fn describe() {
    let tasks: std::sync::RwLockReadGuard<'_, Vec<Task>> = get_tasks().read().unwrap();
    // let functions = get_functions().read().unwrap();

    println!("Task count: {}", tasks.len());
    // println!(
    //     "Functions: {:#?}",
    //     functions.keys().collect::<Vec<&String>>()
    // );
}

pub fn display_hash() {
    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();

    let hash = hash_dag(
        &serde_json::to_string(&*tasks).unwrap(),
        &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
    );
    print!("{hash}");
}

pub fn display_graph(matches: &ArgMatches) {
    let matches = matches.subcommand_matches("graph").unwrap();
    if let Some(subcommand) = matches.get_one::<String>("graph_type") {
        let tasks = get_tasks().read().unwrap();
        let edges = get_edges().read().unwrap();

        let mut runner = InMemoryRunner::new(&tasks, &edges);
        runner.enqueue_run("in_memory", "", Utc::now());

        match subcommand.as_str() {
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
}

pub fn display_tree() {
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
pub fn run_in_memory(dag_path: String, num_threads: usize) {
    // dbg!(1);
    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();

    let mut runner = InMemoryRunner::new(&tasks.to_vec(), &edges);

    let run_id = runner.enqueue_run("", "", Utc::now());
    // dbg!(1);

    let default_tasks = runner.get_default_tasks();
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
    // dbg!(1);

    let (tx, rx) = channel();

    let mut thread_count = 0;

    for _ in 0..num_threads {
        let mut runner = runner.clone();
        let tx = tx.clone();
        let dag_path = dag_path.clone();

        if let Some(queued_task) = runner.pop_priority_queue() {
            thread::spawn(move || {
                let dag_path = Path::new(&dag_path);

                runner.work(run_id, &queued_task, dag_path);
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
    // dbg!(1);
    // let dag_path_2 = Path::new(dag_name.clone());

    for _ in rx.iter() {
        thread_count -= 1;
        // dbg!(2);
        // let dag_path = Path::new(dag_name);
        let dag_path = dag_path.clone();

        let mut runner = runner.clone();
        if let Some(queued_task) = runner.pop_priority_queue() {
            let tx = tx.clone();
            // dbg!(2);

            thread::spawn(move || {

                let dag_path = Path::new(&dag_path);

                runner.work(run_id, &queued_task, dag_path);
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

    // dbg!(1);
}
