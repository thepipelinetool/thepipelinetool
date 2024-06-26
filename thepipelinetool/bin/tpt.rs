use std::{
    env,
    fs::File,
    process::{self, Command},
};

use anyhow::Result;
use clap::Arg;
use thepipelinetool::{
    commands::create_commands, process_subcommands, read_from_executable::read_from_executable,
    read_from_yaml::read_from_yaml, source_type::SourceType,
};
use thepipelinetool_core::dev::{
    assert::assert_operator, params::params_operator, print::print_operator,
    python::python_operator, *,
};
use thepipelinetool_runner::pipeline_options::PipelineOptions;

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("pipeline_source"));
    let matches = command.get_matches();
    let pipeline_source = matches.get_one::<String>("pipeline_source");

    let source_type = SourceType::from_source(pipeline_source);

    let subcommand_name = matches.subcommand_name().unwrap();

    match source_type {
        SourceType::Exe => {
            if args.len() > 4 && args[2..4] == ["run", "function"] {
                let mut cmd = Command::new(pipeline_source.unwrap());
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
            } else {
                read_from_executable(pipeline_source.unwrap())
            }
        }
        SourceType::Yaml => {
            read_from_yaml(serde_yaml::from_reader(File::open(
                pipeline_source.unwrap(),
            )?)?);
        }
        SourceType::Raw => {
            read_from_yaml(serde_json::from_str(pipeline_source.unwrap())?);
        }
        SourceType::None => {
            // try parse operator
            let operator = &serde_json::from_value::<Operator>(json!(args[4])).ok();
            // register built-in operators if used
            if let Some(built_in_operator) = operator {
                _register_function_with_name(
                    match built_in_operator {
                        Operator::BashOperator => bash_operator,
                        Operator::ParamsOperator => params_operator,
                        Operator::PrintOperator => print_operator,
                        Operator::AssertOperator => assert_operator,
                        Operator::PythonOperator => python_operator,
                    },
                    &args[4],
                );
            } else if args[4] == "collector" {
                register_function(collector);
            } else {
                panic!(
                    "no such function '{}'\navailable functions: {:#?}",
                    &args[4],
                    get_functions()
                        .read()
                        .unwrap()
                        .keys()
                        .collect::<Vec<&String>>()
                );
            }

            // for built_in_operator in vec![
            //     Operator::BashOperator,
            //     Operator::ParamsOperator,
            //     Operator::PrintOperator,
            //     Operator::AssertOperator,
            // ] {
            //     _register_function_with_name(
            //         match built_in_operator {
            //             Operator::BashOperator => bash_operator,
            //             Operator::ParamsOperator => params_operator,
            //             Operator::PrintOperator => print_operator,
            //             Operator::AssertOperator => assert_operator,
            //             Operator::PythonOperator => python_operator,
            //         },
            //         &json!(built_in_operator).as_str().unwrap(),
            //     );
            // }
        }
    }

    let options = match source_type {
        SourceType::Exe => PipelineOptions::default(), // TODO read options from exe?
        SourceType::Yaml => serde_yaml::from_reader(File::open(pipeline_source.unwrap())?)?,
        SourceType::Raw => serde_yaml::from_str(pipeline_source.unwrap())?,
        SourceType::None => PipelineOptions::default(),
    };

    let pipeline_path = match source_type {
        SourceType::Exe => pipeline_source.unwrap(),
        SourceType::Yaml => "",
        SourceType::Raw => "",
        SourceType::None => "",
    };

    process_subcommands(pipeline_path, subcommand_name, &options, &matches)?;
    Ok(())
}
