use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: Value) -> Value {
        println!("{}", args);

        json!({
            "hello": "world"
        })
    }

    let mut dag = DAG::new();

    let a = dag.add_task(hi, json!({}), TaskOptions::default());
    let b = dag.add_task(hi, json!({}), TaskOptions::default());
    let _c = dag.add_task(
        hi,
        json!([a.value(), b.get("hello")]),
        TaskOptions {
            timeout: None,
            ..Default::default()
        },
    );

    dag.parse_cli();
}
