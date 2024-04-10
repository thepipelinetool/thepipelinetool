use std::env;
use std::path::Path;

use clap::{arg, command, ArgMatches, Command as CliCommand};

use crate::{
    dev::{execute_function_using_json_files, execute_function_using_json_str_args},
    statics::*,
};

pub fn display_tasks() {
    let tasks = get_tasks().read().unwrap();
    println!("{}", serde_json::to_string_pretty(&*tasks).unwrap());
}

pub fn display_edges() {
    let edges = get_edges().read().unwrap();
    println!("{}", serde_json::to_string_pretty(&*edges).unwrap());
}

pub fn run_function(matches: &ArgMatches) {
    let functions = get_functions().read().unwrap();
    let sub_matches = matches.subcommand_matches("function").unwrap();
    let function_name = sub_matches.get_one::<String>("function_name").unwrap();
    let in_arg = sub_matches.get_one::<String>("in_path").unwrap();
    let out_path_match = sub_matches.get_one::<String>("out_path");

    if functions.contains_key(function_name) {
        if let Some(out_path) = out_path_match {
            execute_function_using_json_files(
                Path::new(in_arg),
                Path::new(out_path),
                &functions[function_name],
            );
        } else {
            execute_function_using_json_str_args(in_arg, &functions[function_name]);
        }
    } else {
        panic!(
            "no such function '{function_name}'\navailable functions: {:#?}",
            functions.keys().collect::<Vec<&String>>()
        )
    }
}

fn create_commands() -> CliCommand {
    command!()
        .about("tpt")
        .subcommand(
            CliCommand::new("describe")
                .about("Describe pipeline tasks or edges")
                .arg_required_else_help(true)
                .subcommand(CliCommand::new("tasks").about("Displays tasks as JSON"))
                .subcommand(CliCommand::new("edges").about("Displays edges as JSON")),
        )
        .subcommand(
            CliCommand::new("run")
                .about("Run complete pipeline or function by name")
                .arg_required_else_help(true)
                .subcommand(
                    CliCommand::new("function")
                        .about("Runs function")
                        .arg(
                            arg!(
                                <function_name> "Function name"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <in_path> "Input file"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <out_path> "Output file"
                            )
                            .required(false),
                        ),
                )
                .subcommand_required(true),
        )
        .subcommand_required(true)
}

///
/// This function parses command-line arguments using the `command!` macro and executes
/// corresponding tasks based on the subcommands and options provided. It interacts with
/// the task management system to perform operations like displaying task information, running
/// tasks, and more.
///
/// The `parse_cli` function is typically called in the `main` function of your Rust application.
/// If you are using the #[dag] macro, it will automatically add a `parse_cli()` function call
/// to the end of the `main` function, simplifying the setup.
/// The behavior of the CLI tool depends on the subcommands and options passed on the command
/// line. Use the "--help" command to see the CLI details.
pub fn parse_cli() {
    let command = create_commands();
    let matches = command.get_matches();
    match matches.subcommand_name().unwrap() {
        "describe" => match matches
            .subcommand_matches("describe")
            .unwrap()
            .subcommand_name()
            .unwrap()
        {
            "tasks" => display_tasks(),
            "edges" => display_edges(),
            _ => {}
        },

        "run" => {
            let matches = matches.subcommand_matches("run").unwrap();
            if matches.subcommand_name().unwrap() == "function" {
                run_function(matches)
            }
        }
        _ => {}
    }
}
