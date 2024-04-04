use serde::{Deserialize, Serialize};
use serde_json::Value;
use thepipelinetool_utils::run_bash_command;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateBashTaskArgs {
    #[serde(default)]
    pub script: String,
}

pub fn bash_operator(args: Value) -> Value {
    let mut command_string = args["_original_command"].as_str().unwrap().to_string();

    for (k, v) in args.as_object().unwrap() {
        if k != "_original_command" {
            command_string = command_string.replace(k, &v.to_string());
        }
    }

    println!("bash_operator$ {}", command_string);
    run_bash_command(&["bash", "-c", &command_string], true, true)
}
