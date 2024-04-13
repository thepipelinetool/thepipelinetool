use std::{
    env,
    fs::File,
    path::Path,
    process::{self, Command},
};

use anyhow::Result;
use clap::Arg;
use thepipelinetool::{
    commands::create_commands, process_subcommands, read_from_executable::read_from_executable,
    read_from_yaml::read_from_yaml,
};
use thepipelinetool_core::dev::*;
use thepipelinetool_runner::pipeline_options::PipelineOptions;

#[derive(Debug, PartialEq)]
enum PipelineSourceType {
    ExeFile,
    YamlFile,
    RawJson,
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("pipeline"));
    let matches = command.get_matches();
    let pipeline_source = matches.get_one::<String>("pipeline").expect("required");
    // let pipeline_path = Path::new(pipeline_name);
    // let pipeline_source = Path::new(pipeline_name);
    let subcommand_name = matches.subcommand_name().unwrap();
    // let is_executable = {
    //     pipeline_path.exists()
    //         && pipeline_path
    //             .extension()
    //             .unwrap_or_default()
    //             .to_str()
    //             .unwrap()
    //             != "yaml"
    // };

    let source_type = {
        let pipeline_path = Path::new(pipeline_source);

        if pipeline_path.exists() {
            if pipeline_path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap()
                == "yaml"
            {
                PipelineSourceType::YamlFile
            } else {
                PipelineSourceType::ExeFile
            }
        } else {
            PipelineSourceType::RawJson
        }
    };

    // passthrough 'run function' commands if pipeline is executable
    let options = if source_type == PipelineSourceType::ExeFile
        && args.len() > 4
        && args[2..4] == ["run", "function"]
    {
        let mut cmd = Command::new(pipeline_source);
        cmd.args(&mut args[2..]);
        let exit_status = spawn(
            cmd,
            None,
            Box::new(|x| {
                print!("{x}");
                Ok(())
            }),
            Box::new(|x| {
                eprint!("{x}");
                Ok(())
            }),
        )?;
        process::exit(exit_status.code().unwrap());
    } else if source_type == PipelineSourceType::ExeFile {
        read_from_executable(pipeline_source);
        PipelineOptions::default()
    } else if source_type == PipelineSourceType::YamlFile {
        // TODO enable flag to load from binary as well when reading YAML pipeline
        let value: Value = serde_yaml::from_reader(File::open(pipeline_source).unwrap()).unwrap();
        let options: PipelineOptions = serde_json::from_value(value.clone())?;
        read_from_yaml(value);
        options
    } else {
        let value: Value = serde_yaml::from_str(&pipeline_source)?;
        let options: PipelineOptions = serde_json::from_value(value.clone())?;

        read_from_yaml(value);
        options
    };

    process_subcommands(
        pipeline_source,
        // pipeline_source,
        subcommand_name,
        &options,
        &matches,
    );
    Ok(())
}
