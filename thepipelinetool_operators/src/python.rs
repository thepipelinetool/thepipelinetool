use serde::{Deserialize, Serialize};
use serde_json::Value;
use thepipelinetool_utils::run_bash_command;

use crate::ORIGINAL_STRING_KEY;

pub const REQUIREMENTS_KEY: &str = "_requirements";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplatePythonArgs {
    #[serde(default)]
    pub requirements: Vec<String>,

    #[serde(default)]
    pub script: String,
}

pub fn python_operator(args: Value) -> Value {
    let mut command_string = args[ORIGINAL_STRING_KEY].as_str().unwrap().to_string();

    for (k, v) in args.as_object().unwrap() {
        if k != ORIGINAL_STRING_KEY && k != REQUIREMENTS_KEY {
            command_string = command_string.replace(k, &v.to_string());
        }
    }
    let formatted_command_string = format!("python3 - <<-EOF\n{command_string}\nEOF");

    for requirement in args[REQUIREMENTS_KEY].as_array().unwrap() {
        run_bash_command(
            &["pip3", "install", &requirement.as_str().unwrap()],
            true,
            false,
        );
    }

    // println!("python_operator$\n{}", command_string);
    run_bash_command(&["bash", "-c", &formatted_command_string], false, true)
}
