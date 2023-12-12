use thepipelinetool::prelude::*;

fn produce_data(_: ()) -> String {
    "world".to_string()
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[dag]
fn main() {
    // add a task that uses the function 'produce_data'
    // let task_ref = add_task(produce_data, (), opts);
    let task_ref = TaskBuilder::new().build(produce_data, ());

    // add a task that depends on 'task_ref'
    let _ = TaskBuilder::new().build_with_ref(print_data, &task_ref);
}
