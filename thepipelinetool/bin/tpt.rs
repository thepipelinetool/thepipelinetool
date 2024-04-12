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
use thepipelinetool_core::dev::{params::params_operator, print::print_operator, *};
use thepipelinetool_runner::pipeline_options::PipelineOptions;

#[derive(Debug, PartialEq)]
enum PipelineSourceType {
    None,
    ExeFile,
    YamlFile,
    RawJson,
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("pipeline"));
    let matches = command.get_matches();
    let pipeline_source = matches.get_one::<String>("pipeline");
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

    // for operator in Operator
    //  {
    //     _register_function_with_name(operator, &function_name_as_string(operator));
    // }

    let source_type = {
        if let Some(pipeline_source) = pipeline_source {
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
        } else {
            PipelineSourceType::None
        }
    };

    // passthrough 'run function' commands if pipeline is executable
    let options = if source_type == PipelineSourceType::ExeFile
        && args.len() > 4
        && args[2..4] == ["run", "function"]
    {
        let mut cmd = Command::new(pipeline_source.expect(""));
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
        read_from_executable(pipeline_source.expect(""));
        Some(PipelineOptions::default())
    } else if source_type == PipelineSourceType::YamlFile {
        // TODO enable flag to load from binary as well when reading YAML pipeline
        let value: Value =
            serde_yaml::from_reader(File::open(pipeline_source.expect("")).unwrap()).unwrap();
        let options: PipelineOptions = serde_json::from_value(value.clone())?;
        read_from_yaml(value);
        Some(options)
    } else if source_type == PipelineSourceType::RawJson {
        let value: Value = serde_yaml::from_str(&pipeline_source.expect(""))?;
        let options: PipelineOptions = serde_json::from_value(value.clone())?;

        read_from_yaml(value);
        Some(options)
    } else {
        if args[1..3] == ["run", "function"] {
            // try parse operator
            let operator_name = &args[3];
            let operator = &serde_json::from_value::<Operator>(json!(operator_name)).ok();
            dbg!(&operator_name);
            // register built-in operators if used
            if let Some(built_in_operator) = operator {
                _register_function_with_name(
                    match built_in_operator {
                        Operator::BashOperator => bash_operator,
                        Operator::ParamsOperator => params_operator,
                        Operator::PrintOperator => print_operator,
                        Operator::CollectorOperator => collector_operator,
                    },
                    operator_name,
                );
            }
        }
        None
    };

    process_subcommands(
        // pipeline_source.unwrap(),
        // pipeline_source,
        subcommand_name,
        options,
        &matches,
    );
    Ok(())
}
