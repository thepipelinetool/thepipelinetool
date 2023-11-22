use thepipelinetool::prelude::*;

fn branch_task(_: Value) -> Branch<usize> {
    Branch::Left(0)
}

fn left(arg: usize) -> () {
    println!("left {arg}");
}

pub fn right(_: usize) -> () {
    println!("right");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_right() {
        right(1);
    }
}

// #[dag]
fn main() {
    // add_task!(right, 0);
    // println!("{}", serde_json::to_string(&branch_task(Value::Null)).unwrap());
    let _ = branch(branch_task, json!({}), left, right, &TaskOptions::default());
    dbg!(1);
    // run_in_memory(1);
}
