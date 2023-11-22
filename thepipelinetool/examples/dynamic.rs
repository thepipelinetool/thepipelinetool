use thepipelinetool::prelude::*;

fn produce_lazy(_: ()) -> Vec<u8> {
    vec![0, 1]
}

fn say_hello(arg: u8) -> u8 {
    println!("hello {arg}");
    arg
}

#[dag]
fn main() {
    let opts = &TaskOptions::default();

    let produce_lazy_task_ref = add_task(produce_lazy, (), opts);

    // creates a new task for each item in 'produce_lazy' result
    let expanded_lazy_task_ref = expand_lazy(say_hello, &produce_lazy_task_ref, opts);

    // you can also chain lazily expanded tasks
    let _ = expand_lazy(say_hello, &expanded_lazy_task_ref, opts);
}