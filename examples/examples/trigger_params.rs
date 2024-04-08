use thepipelinetool_core::prelude::*;

#[derive(Serialize, Deserialize)]
struct Params {
    data: String,
}

fn print_trigger_params(params: Params) -> () {
    println!("{}", params.data);
}

#[dag]
fn main() {
    let opts = &TaskOptions::default();

    let _ = add_task_using_trigger_params(print_trigger_params, opts);
}
