use thepipelinetool_core::{prelude::*, tpt};

fn branch_task(_: Value) -> Branch<usize> {
    Branch::Left(0)
}

fn left(arg: usize) -> () {
    println!("left {arg}");
}

fn right(_: usize) -> () {
    println!("right");
}

#[tpt::main]
fn main() {
    let _ = branch(branch_task, json!({}), left, right, &TaskOptions::default());
}
