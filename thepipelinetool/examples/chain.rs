use thepipelinetool::prelude::*;

fn main() {
    fn hi(_args: Value) -> Value {
        println!("hi");

        json!({
            "hello": "world"
        })
    }

    let mut dag = DAG::new("chain");

    let a = dag.add_task(hi, json!({}), TaskOptions::default());
    let b = dag.add_task(hi, json!({}), TaskOptions::default());
    let c = dag.add_task(hi, json!({}), TaskOptions::default());

    dag.seq(&[&c, &b, &a]);

    let _d = dag.add_task(hi, json!({}), TaskOptions::default());
    let _e = dag.add_task(hi, json!({}), TaskOptions::default());

    // let _ = TaskGroup(vec![d, e]) >> TaskGroup(vec![a]);

    let _out = dag.expand(
        hi,
        &[Value::Null, Value::Null, Value::Null],
        TaskOptions::default(),
    );

    dag.parse_cli();
}
