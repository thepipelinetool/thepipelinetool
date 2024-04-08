use clap::{arg, command, value_parser, Command as CliCommand};
use serde_json::Value;

pub fn create_commands() -> CliCommand {
    command!()
        .about("tpt")
        .subcommand(
            CliCommand::new("describe")
                .about("Run complete DAG or function by name")
                .arg_required_else_help(true)
                .subcommand(CliCommand::new("tasks").about("Displays tasks as JSON"))
                .subcommand(CliCommand::new("edges").about("Displays edges as JSON"))
                .subcommand(CliCommand::new("hash").about("Displays hash as JSON"))
                .subcommand(CliCommand::new("options").about("Displays options as JSON")),
        )
        .subcommand(CliCommand::new("check").about("Check for circular depencencies"))
        .subcommand(
            CliCommand::new("graph")
                .about("Displays graph")
                .arg_required_else_help(true)
                .arg(
                    arg!(
                        [graph_type] "Type of graph to output"
                    )
                    .required(true)
                    .value_parser(value_parser!(String))
                    .default_values(["mermaid", "graphite"])
                    .default_missing_value("mermaid"),
                ),
        )
        .subcommand(CliCommand::new("tree").about("Displays tree"))
        .subcommand(
            CliCommand::new("run")
                .about("Run complete DAG or function by name")
                .arg_required_else_help(true)
                .subcommand(
                    CliCommand::new("in_memory")
                        .about("Runs this DAG in memory")
                        .arg(
                            arg!(
                                --max_parallelism <max_parallelism> "Max number of threads for parallel execution"
                            )
                            .required(false)
                            .value_parser(value_parser!(String))
                            .default_value("max")
                            .default_missing_value("max"),
                        )
                        .arg(
                            arg!(
                                --params <params> "Trigger params"
                            )
                            .required(false)
                            .value_parser(value_parser!(String))
                            .default_value("")
                            .default_missing_value(""),
                        ),
                )
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
