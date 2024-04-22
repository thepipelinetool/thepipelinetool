use std::{thread::sleep, time::Duration};

use thepipelinetool_core::{prelude::*, tpt};

fn produce_data(_: ()) -> String {
    "world".to_string()
}

fn produce_data2(_: ()) -> String {
    sleep(Duration::new(3, 0));
    "world".to_string()
}

fn print_data(_: ()) -> () {
    println!("hello");
}

#[tpt::main]
fn main() {
    let opts = &TaskOptions::default();

    // add a task that uses the function 'produce_data'
    let task_ref0 = add_task(produce_data, (), opts);
    let task_ref1 = add_task(produce_data2, (), opts);

    let mut opts = TaskOptions::default();
    opts.trigger_rule = TriggerRule::AnyDone;
    let task_ref2 = add_task(print_data, (), &opts);

    let _ = task_ref0 >> &task_ref2;
    let _ = task_ref1 >> &task_ref2;
}
