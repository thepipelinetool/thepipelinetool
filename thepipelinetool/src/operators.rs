use std::process::Command;

use serde_json::{json, Value};

pub fn run_command(args: Value) -> Value {
    let mut args: Vec<&str> = args
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let output = Command::new(args[0])
        .args(&mut args[1..])
        .output()
        .unwrap_or_else(|_| panic!("failed to run command:\n{}\n\n", args.join(" ")));
    let result_raw = String::from_utf8_lossy(&output.stdout);
    let err_raw = String::from_utf8_lossy(&output.stderr);

    println!("{}", result_raw);
    if !output.status.success() {
        eprintln!("{}", err_raw);
        panic!("failed to run command:\n{}\n\n", args.join(" "));
    }

    json!(result_raw.to_string().trim_end())
}
