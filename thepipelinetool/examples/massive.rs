use thepipelinetool::prelude::*;

fn hi(args: usize) -> Value {
    println!("{}", args);

    json!({
        "hello": "world"
    })
}

#[dag]
fn main() {
    let binding = (0..500).collect::<Vec<usize>>();
    let k: &[usize; 500] = binding.as_slice().try_into().unwrap();
    let _ = expand(hi, k, &TaskOptions::default());
}
