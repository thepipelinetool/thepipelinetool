use thepipelinetool::prelude::*;

#[derive(Deserialize, Serialize)]
struct MyConfig {
    data: String,
}

fn produce_data(config: MyConfig) -> String {
    config.data
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[dag]
fn main() {
    // define a task that uses the function 'produce_data'
    let task_ref = add_task(produce_data, MyConfig{ data: "world".into() }, &TaskOptions::default());

    // this task will wait use the result from produce_data
    let _ = add_task_with_ref(print_data, &task_ref, &TaskOptions::default());
}