use thepipelinetool::prelude::*;

fn branch_task(_: Value) -> Branch<usize> {
    Branch::left(0)
}

fn left(arg: usize) -> () {
    println!("left {arg}");
}

fn right(_: usize) -> () {
    println!("right");
}

#[dag]
fn main() {
    let _ = branch(branch_task, json!({}), left, right, &TaskOptions::default());
}
