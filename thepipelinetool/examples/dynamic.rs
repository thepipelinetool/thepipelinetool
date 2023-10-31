use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: u8) -> u8 {
        println!("{}", args);
        args
    }

    fn hi2(args: Value) -> [u8; 2] {
        println!("{}", args);

        [0, 1]
    }

    let mut dag = DAG::new();

    let a = dag.add_task(hi2, json!({}), TaskOptions::default());
    let b = dag.expand_lazy(hi, &a, TaskOptions::default());
    // let c = dag.add_task(hi2, json!({}), TaskOptions::default());

    // dag.seq(&[&b, &c]);

    dag.expand_lazy(hi, &b, TaskOptions::default());

    dag.parse_cli();
    // println!("{}", dag.get_mermaid_graph());
}
