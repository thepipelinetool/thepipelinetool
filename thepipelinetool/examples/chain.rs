use thepipelinetool::prelude::*;

#[dag]
fn main() {
    fn hi(_args: Value) -> Value {
        println!("hi");

        json!({
            "hello": "world"
        })
    }

    fn hi2(_args: Value) -> Value {
        println!("hi");

        json!({
            "hello": "world"
        })
    }

    let a = add_task(hi, json!({}), &TaskOptions::default());
    let b = add_task(hi, json!({}), &TaskOptions::default());
    let c = add_task(hi, json!({}), &TaskOptions::default());

    let d = add_task(hi2, json!({}), &TaskOptions::default());
    let e = add_task(hi2, json!({}), &TaskOptions::default());
    let f = add_task(hi2, json!({}), &TaskOptions::default());

    // let p = par(&d, &e);
    // let p = ;
    // seq(&[&c, &b, &p, &a]);
    // println!("{:#?}", p.0.task_ids);
    let _ = b >> (d | e | c) >> a >> f;

    let _out = expand(
        hi,
        &[Value::Null, Value::Null, Value::Null],
        &TaskOptions::default(),
    );

    // println!("{}", get_initial_mermaid_graph());
}
