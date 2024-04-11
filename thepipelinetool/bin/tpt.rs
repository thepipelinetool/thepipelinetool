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
use thepipelinetool_runner::pipeline_options::PipelineOptions;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("pipeline"));
    let matches = command.get_matches();
    let pipeline_name = matches.get_one::<String>("pipeline").expect("required");
    let pipeline_path = Path::new(pipeline_name);
    let subcommand_name = matches.subcommand_name().unwrap();
    let is_executable = pipeline_path.extension().unwrap_or_default().to_str().unwrap() != "yaml";

    // passthrough 'run function' commands if pipeline is executable
    if is_executable && args.len() > 4 && args[2..4] == ["run", "function"] {
        let mut cmd = Command::new(pipeline_path);
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
        read_from_executable(pipeline_name)
    } else {
        // TODO enable flag to load from binary as well when reading YAML pipeline

        read_from_yaml(pipeline_path);
    }

    let options: PipelineOptions = match File::open(pipeline_path.with_extension("yaml")) {
        Ok(file) => serde_yaml::from_reader(file).unwrap_or_default(),
        Err(_) => PipelineOptions::default(),
    };

    process_subcommands(pipeline_path, pipeline_name, subcommand_name, &options, &matches);
}
