use thepipelinetool_core::prelude::*;

fn print_trigger_params(arg: Value) -> () {
    println!("{}", arg);
}

#[dag]
fn main() {
    let opts = &TaskOptions::default();

    let _ = add_task_using_trigger_params(print_trigger_params, opts);
}
