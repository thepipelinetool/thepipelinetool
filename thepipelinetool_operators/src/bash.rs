use serde::{Deserialize, Serialize};
use serde_json::Value;
use thepipelinetool_utils::run_bash_command;

use crate::ORIGINAL_STRING_KEY;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateBashTaskArgs {
    #[serde(default)]
    pub script: String,
}

pub fn bash_operator(args: Value) -> Value {
    let mut command_string = args[ORIGINAL_STRING_KEY].as_str().unwrap().to_string();

    for (k, v) in args.as_object().unwrap() {
        if k != ORIGINAL_STRING_KEY {
            command_string = command_string.replace(k, &v.to_string());
        }
    }

    println!("bash_operator$ {}", command_string);
    run_bash_command(&["bash", "-c", &command_string], true, true)
}
