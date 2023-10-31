use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: Value) -> Value {
        println!("{}", args);

        json!({
            "hello": "world"
        })
    }

    fn hi2(args: Value) -> Value {
        println!("{}", args);

        json!(["hello", "world"])
    }

    let mut dag = DAG::new("direct");

    let a = dag.add_task(hi, json!({}), TaskOptions::default());
    // let b = dag.add_task(hi, json!({}), TaskOptions::default());
    // let c = dag.add_task(hi, json!({}), TaskOptions::default());

    // dag.chain(vec![&c, &b, &a]);

    // let d = dag.add_task(hi, json!({}), TaskOptions::default());
    // let e = dag.add_task(hi, json!({}), TaskOptions::default());

    // // let _ = TaskGroup(vec![d, e]) >> TaskGroup(vec![a]);

    // let _out = dag.expand(
    //     hi,
    //     vec![Value::Null, Value::Null, Value::Null],
    //     TaskOptions::default(),
    // );

    let _ = dag.add_task_with_ref(hi, &a, TaskOptions::default());
    let _ = dag.add_task(hi, a.get("hello"), TaskOptions::default());
    let b = dag.add_task_with_ref(hi2, &a, TaskOptions::default());
    let _ = dag.add_task_with_ref(hi2, &b, TaskOptions::default());
    // dag.add_task(dag.functions[""].clone(), b.value(), TaskOptions::default());
    // println!("{}", dag.get_mermaid_graph());

    dag.parse_cli();
}
