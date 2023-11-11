use thepipelinetool::prelude::*;

fn main() {
    fn brnch(_: Value) -> Branch<usize> {
        Branch::left(0)
    }

    fn left(arg: usize) -> () {
        println!("left {arg}");
    }

    fn right(_: usize) -> () {
        println!("right");
    }

    let _a = branch(brnch, json!({}), left, right, TaskOptions::default());

    parse_cli();
}
