use serde_json::Value;
use thepipelinetool_utils::run_bash_commmand;

pub fn bash_operator(args: Value) -> Value {
    run_bash_commmand(
        &args
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_str().unwrap())
            .collect(),
        false,
    )
}
