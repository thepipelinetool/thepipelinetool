use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: usize) -> Value {
        println!("{}", args);

        json!({
            "hello": "world"
        })
    }

    // let a = add_task(hi, vec, &TaskOptions::default());

    let binding = (0..500).collect::<Vec<usize>>();
    let k: &[usize; 500] = binding.as_slice().try_into().unwrap();
    let _ = expand(hi, k, &TaskOptions::default());

    parse_cli();
}
