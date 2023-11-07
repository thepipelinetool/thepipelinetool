use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: usize) -> Value {
        println!("{}", args);

        json!({
            "hello": "world"
        })
    }

    let mut dag = DAG::new();

    // let a = dag.add_task(hi, vec, TaskOptions::default());

    let binding = (0..500).collect::<Vec<usize>>();
    let k: &[usize; 500] = binding.as_slice().try_into().unwrap();
    let _ = dag.expand(hi, k, TaskOptions::default());

    dag.parse_cli();
}
