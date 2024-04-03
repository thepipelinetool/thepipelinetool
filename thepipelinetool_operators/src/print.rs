use serde_json::Value;
use thepipelinetool_utils::run_bash_command;

pub fn print_operator(value: Value) -> Value {
    run_bash_command(&["echo", &format!("{value}")], false, false);
    value
}
