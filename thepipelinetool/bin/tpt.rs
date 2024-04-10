use std::{
    env,
    fs::File,
    path::Path,
    process::{self, Command},
};

use clap::Arg;
use thepipelinetool::{
    commands::create_commands, process_subcommands, read_from_executable::read_from_executable,
    read_from_yaml::read_from_yaml,
};
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::options::DagOptions;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("dag"));
    let matches = command.get_matches();
    let dag_name = matches.get_one::<String>("dag").expect("required");
    let dag_path = Path::new(dag_name);
    let subcommand_name = matches.subcommand_name().unwrap();
    let is_executable = dag_path.extension().unwrap_or_default().to_str().unwrap() != "yaml";

    // passthrough 'run function' commands if pipeline is executable
    if is_executable && args.len() > 4 && args[2..4] == ["run", "function"] {
        let mut cmd = Command::new(dag_path);
        cmd.args(&mut args[2..]);
        let (exit_status, _) = spawn(
            cmd,
            Box::new(|x| {
                print!("{x}");
                Ok(())
            }),
            Box::new(|x| {
                eprint!("{x}");
                Ok(())
            }),
        );
        process::exit(exit_status.code().unwrap());
    } else if is_executable {
        read_from_executable(dag_name)
    } else {
        // TODO enable flag to load from binary as well when reading YAML pipeline

        read_from_yaml(dag_path);
    }

    let options: DagOptions = match File::open(dag_path.with_extension("yaml")) {
        Ok(file) => serde_yaml::from_reader(file).unwrap_or_default(),
        Err(_) => DagOptions::default(),
    };

    process_subcommands(dag_path, dag_name, subcommand_name, &options, &matches);
}
