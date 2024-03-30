use std::{
    env,
    fs::File,
    path::Path,
    process::{self, Command},
};

use clap::Arg;
use thepipelinetool::dev::*;
use thepipelinetool_cli::{create_commands, load_from_binary, process_subcommands, yaml::read_from_yaml};
use thepipelinetool_runner::options::DagOptions;

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

            load_from_binary(dag_name);
        }
        DagType::YAML => {
            // TODO read from flag to enable load_from_binary
            // load_from_binary(dag_name);

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
                    } // Operator::Papermill => {
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
    let options: DagOptions = match File::open(dag_path.with_extension("yaml")) {
        Ok(file) => match serde_yaml::from_reader(file) {
            Ok(res) => res,
            Err(_) => DagOptions::default(),
        },
        Err(_) => DagOptions::default(),
    };

    // {
    //     let default = DagOptions::default();

    //     let open_file_result = File::open(dag_path.with_extension("yaml"));

    //     if open_file_result.is_err() {
    //         return default;
    //     }
    //     let file = open_file_result.unwrap();

    //     let read_result = serde_yaml::from_reader(file);

    //     if read_result.is_err() {
    //         return default;
    //     }

    //     read_result.unwrap()
    // };

    let tasks = get_tasks().read().unwrap();
    let edges = get_edges().read().unwrap();
    process_subcommands(
        dag_path,
        dag_name,
        subcommand_name,
        &tasks,
        &edges,
        &options,
        &matches,
    );
}
