use std::{
    env,
    path::Path,
    process::{self, Command},
};

use clap::Arg;
use serde_json::json;
use thepipelinetool::dev::*;
use thepipelinetool_cli::{create_commands, process_subcommands};
use thepipelinetool_reader::yaml::read_from_yaml;

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
