use std::collections::HashMap;

use serde_json::{json, Value};
use thepipelinetool_utils::run_bash_command;

pub fn bash_operator(args: Value) -> Value {
    let mut command_string = args["_original_command"].as_str().unwrap().to_string();

    // let args: Vec<String> = args
    //     .as_array()
    //     .unwrap()
    //     .iter()
    //     .map(|e| match e.as_str() {
    //         Some(e) => e.to_string(),
    //         None => e.to_string(),
    //     })
    //     .collect();
    // .map(|e| match e.as_str() {
    //     Some(e) => match serde_json::from_str::<Value>(e) {
    //         Ok(_) => todo!(),
    //         Err(_) => e.to_string(),
    //         // _ => e.to_string(),
    //     },
    //     None => e.to_string(),
    // })
    // .collect();
    for (k, v) in args.as_object().unwrap() {
        if k != "_original_command" {
            command_string = command_string.replace(k, &v.to_string());
        }
    }

    println!("bash_operator$ {}", command_string);
    run_bash_command(&vec!["bash", "-c", &command_string], true, true)
}
