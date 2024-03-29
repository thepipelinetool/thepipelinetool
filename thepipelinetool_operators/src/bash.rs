use serde_json::Value;
use thepipelinetool_utils::run_bash_commmand;

pub fn bash_operator(args: Value) -> Value {
    let args: Vec<String> = args
        .as_array()
        .unwrap()
        .iter()
        .map(|e| match e.as_str() {
            Some(e) => e.to_string(),
            None => e.to_string(),
        })
        .collect();
    // .map(|e| match e.as_str() {
    //     Some(e) => match serde_json::from_str::<Value>(e) {
    //         Ok(_) => todo!(),
    //         Err(_) => e.to_string(),
    //         // _ => e.to_string(),
    //     },
    //     None => e.to_string(),
    // })
    // .collect();
    println!("bash_operator$ {}", args.join(" "));
    run_bash_commmand(
        &args.iter().map(|f| f.as_str()).collect::<Vec<&str>>(),
        true,
    )
}
