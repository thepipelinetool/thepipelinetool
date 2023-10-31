use thepipelinetool::prelude::*;

fn main() {
    fn hi(_args: Value) -> Value {
        println!("hi");

        json!({
            "hello": "world"
        })
    }

    let mut dag = DAG::new("chain_mermaid");

    let a = dag.add_task(hi, json!({}), TaskOptions::default());
    let b = dag.add_task(hi, json!({}), TaskOptions::default());
    let c = dag.add_task(hi, json!({}), TaskOptions::default());

    let d = dag.add_task(hi, json!({}), TaskOptions::default());
    let e = dag.add_task(hi, json!({}), TaskOptions::default());

    let p = &dag.par(&[&d, &e]);
    dag.seq(&[&c, &b, &p, &a]);

    let _out = dag.expand(
        hi,
        &[Value::Null, Value::Null, Value::Null],
        TaskOptions::default(),
    );

    dag.parse_cli();
    // println!("{}", dag.get_initial_mermaid_graph());
}
