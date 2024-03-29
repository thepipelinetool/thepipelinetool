use std::{
    env,
    path::Path,
    process::{self, Command},
};

use clap::Arg;
use thepipelinetool::dev::*;
use thepipelinetool_cli::{
    create_commands, process_subcommands,
    yaml::read_from_yaml,
};

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("dag"));
    let matches = command.get_matches();
    let dag_name = matches.get_one::<String>("dag").expect("required");
    let dag_path = Path::new(dag_name);

    let subcommand_name = matches.subcommand_name().unwrap();

    match get_dag_type_by_path(dag_path.to_path_buf()) {
        DagType::Binary => {
            if args.len() > 4 && args[2..4] == ["run", "function"] {
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
                let tasks_from_json: Vec<Task> = serde_json::from_str(
                    run_bash_commmand(&vec![dag_name, "tasks"], true)
                        .as_str()
                        .unwrap(),
                )
                .unwrap();

                for task in tasks_from_json {
                    get_tasks().write().unwrap().insert(task.id, task);
                }
            }

            if load_edges {
                let edges_from_json: Vec<(usize, usize)> = serde_json::from_str(
                    run_bash_commmand(&vec![dag_name, "edges"], true)
                        .as_str()
                        .unwrap(),
                )
                .unwrap();

                for edge in edges_from_json {
                    get_edges().write().unwrap().insert(edge);
                }
            }
        }
        DagType::YAML => {
            let (template_tasks, edges) = read_from_yaml(dag_path);
            for template_task in template_tasks {
                match template_task.operator {
                    Operator::Bash => {
                        add_named_task(
                            bash_operator,
                            template_task.args,
                            &template_task.options,
                            &template_task.name,
                        );
                    }
                    // Operator::Papermill => {
                    //     add_named_task(
                    //         papermill_operator,
                    //         serde_json::from_value(template_task.args).unwrap(),
                    //         &template_task.options,
                    //         &template_task.name,
                    //     );
                    // }
                }
            }
            for edge in edges {
                get_edges().write().unwrap().insert(edge);
            }
        }
    }

    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();
    process_subcommands(&dag_name, subcommand_name, &tasks, &edges, &matches);
}
