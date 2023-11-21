use thepipelinetool::prelude::*;

fn hi(args: u8) -> u8 {
    println!("{}", args);
    args
}

fn hi2(args: Value) -> [u8; 2] {
    println!("{}", args);

    [0, 1]
}

#[dag]
fn main() {
    let a = add_task(hi2, json!({}), &TaskOptions::default());
    let b = expand_lazy(hi, &a, &TaskOptions::default());

    expand_lazy(hi, &b, &TaskOptions::default());
}
