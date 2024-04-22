use thepipelinetool_core::{prelude::*, tpt};

fn produce_data(_: ()) -> String {
    "world".to_string()
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

fn taska(_: ()) -> String {
    "world".to_string()
}

#[tpt::main]
fn main() {
    let opts = &TaskOptions::default();

    // add a task that uses the function 'produce_data'
    let task_ref = add_task(produce_data, (), opts);

    // add a task that depends on 'task_ref'
    let a = add_task_with_ref(print_data, &task_ref, opts);

    let b = add_task_with_ref(taska, &a, opts);

    let _ = b >> task_ref;
}
