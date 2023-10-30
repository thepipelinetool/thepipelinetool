use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: usize) -> Value {
        println!("{}", args);

        json!({
            "hello": "world"
        })
    }

    let mut dag = DAG::new("simple");

    // let a = dag.add_task(hi, vec, TaskOptions::default());

    let binding = (0..10000).collect::<Vec<usize>>();
    let k: &[usize; 10000] = binding.as_slice().try_into().unwrap();
    let _ = dag.expand(hi, k, TaskOptions::default());

    dag.apply_cli_args();
}
