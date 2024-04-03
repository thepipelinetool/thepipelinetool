use std::{
    env,
    fs::File,
    path::Path,
    process::{self, Command},
};

use clap::Arg;
use thepipelinetool::{
    binary::load_from_binary, commands::create_commands, get_dag_type_by_path, process_subcommands,
    yaml::read_from_yaml, DagType,
};
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::options::DagOptions;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("dag"));
    let matches = command.get_matches();
    let dag_name = matches.get_one::<String>("dag").expect("required");
    let dag_path = Path::new(dag_name);
    let binary_path = dag_path.with_extension("");

    let subcommand_name = matches.subcommand_name().unwrap();
    if binary_path.exists() {
        load_from_binary(binary_path.to_str().unwrap());
    }

    // load built-in operators
    // for operator in [bash_operator] {
    //     let function_name = function_name_as_string(&operator).to_string();

    //     get_functions()
    //         .write()
    //         .unwrap()
    //         .insert(function_name, Box::new(wrap_function(operator)));
    // }

    match get_dag_type_by_path(dag_path.to_path_buf()) {
        DagType::Binary => {
            if !binary_path.exists() {
                eprintln!("input dag does not exist");
                process::exit(-1);
            }
        }
        DagType::YAML => read_from_yaml(dag_path),
    }

    if binary_path.exists() && args.len() > 4 && args[2..4] == ["run", "function"] {
        let mut cmd = Command::new(binary_path);
        cmd.args(&mut args[2..]);
        let (exit_status, _) = spawn(
            cmd,
            Box::new(|x| print!("{x}")),
            Box::new(|x| eprint!("{x}")),
        );
        process::exit(exit_status.code().unwrap());
    }

    let options: DagOptions = match File::open(dag_path.with_extension("yaml")) {
        Ok(file) => match serde_yaml::from_reader(file) {
            Ok(res) => res,
            Err(_) => DagOptions::default(),
        },
        Err(_) => DagOptions::default(),
    };

    process_subcommands(dag_path, dag_name, subcommand_name, &options, &matches);
}
