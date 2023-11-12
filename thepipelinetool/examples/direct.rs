use thepipelinetool::prelude::*;

#[dag]
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

    let a = add_task(hi, json!({}), &TaskOptions::default());
    // let b = add_task(hi, json!({}), &TaskOptions::default());
    // let c = add_task(hi, json!({}), &TaskOptions::default());

    // chain(vec![&c, &b, &a]);

    // let d = add_task(hi, json!({}), &TaskOptions::default());
    // let e = add_task(hi, json!({}), &TaskOptions::default());

    // // let _ = TaskGroup(vec![d, e]) >> TaskGroup(vec![a]);

    // let _out = expand(
    //     hi,
    //     vec![Value::Null, Value::Null, Value::Null],
    //     TaskOptions::default(),
    // );

    let _ = add_task_with_ref(hi, &a, &TaskOptions::default());
    let _ = add_task_with_ref(hi, &a.get("hello"), &TaskOptions::default());
    let b = add_task_with_ref(hi2, &a, &TaskOptions::default());
    let _ = add_task_with_ref(hi2, &b, &TaskOptions::default());
    // add_task(functions[""].clone(), b.value(), &TaskOptions::default());
    // println!("{}", get_mermaid_graph());
}
