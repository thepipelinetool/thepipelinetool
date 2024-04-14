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

#[derive(PartialEq, Eq)]
enum SourceType {
    Exe,
    Yaml,
    Raw,
}

impl SourceType {
    pub fn from_source(source: &str) -> Self {
        let p = Path::new(source);
        if p.exists() {
            match p.extension() {
                Some(ext) => match ext.to_str().unwrap() {
                    "yaml" => SourceType::Yaml,
                    _ => panic!("unknown extenstion type"),
                },
                None => SourceType::Exe,
            }
        } else {
            SourceType::Raw
        }
    }
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    let command = create_commands().arg(Arg::new("pipeline_source").required(true));
    let matches = command.get_matches();
    let pipeline_source = matches
        .get_one::<String>("pipeline_source")
        .expect("required");

    let source_type = SourceType::from_source(pipeline_source);

    let subcommand_name = matches.subcommand_name().unwrap();

    match source_type {
        SourceType::Exe => {
            if args.len() > 4 && args[2..4] == ["run", "function"] {
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
            } else {
                read_from_executable(pipeline_source)
            }
        }
        SourceType::Yaml => {
            read_from_yaml(serde_yaml::from_reader(File::open(pipeline_source)?)?);
        }
        SourceType::Raw => {
            read_from_yaml(serde_json::from_str(pipeline_source)?);
        }
    }

    let options = match source_type {
        SourceType::Exe => {
            // TODO
            PipelineOptions::default()
        }
        SourceType::Yaml => serde_yaml::from_reader(File::open(pipeline_source)?)?,
        SourceType::Raw => serde_yaml::from_str(pipeline_source)?,
    };

    let pipeline_path = match source_type {
        SourceType::Exe => pipeline_source,
        SourceType::Yaml => pipeline_source,
        SourceType::Raw => &args[0],
    };

    process_subcommands(
        pipeline_path,
        // pipeline_source,
        subcommand_name,
        &options,
        &matches,
    )?;
    Ok(())
}
