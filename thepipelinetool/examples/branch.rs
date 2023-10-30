use thepipelinetool::prelude::*;

fn main() {
    fn branch(_: Value) -> Branch<usize> {
        Branch::left(0)
    }

    fn left(arg: usize) -> () {
        println!("left {arg}");
    }

    fn right(_: usize) -> () {
        println!("right");
    }

    let mut dag = DAG::new("simple");

    let _a = dag.branch(branch, json!({}), left, right, TaskOptions::default());

    dag.apply_cli_args();
}
