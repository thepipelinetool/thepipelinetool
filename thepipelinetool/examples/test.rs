use thepipelinetool::prelude::*;

fn branch_task(_: Value) -> Branch<usize> {
    Branch::left(0)
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

#[dag]
fn main() {
    let _ = branch(branch_task, json!({}), left, right, &TaskOptions::default());
}
