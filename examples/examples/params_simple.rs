use thepipelinetool_core::{prelude::*, tpt};

#[derive(Serialize, Deserialize)]
struct Params {
    data: String,
}

fn print_trigger_params(params: Params) -> () {
    println!("{}", params.data);
}

#[tpt::main]
fn main() {
    let opts = &TaskOptions::default();

    let _ = add_task_using_trigger_params(print_trigger_params, opts);
}
