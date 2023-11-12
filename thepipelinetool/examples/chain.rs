use thepipelinetool::prelude::*;

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

#[dag]
fn main() {
    let a = add_task(hi, json!({}), &TaskOptions::default());
    let b = add_task(hi, json!({}), &TaskOptions::default());
    let c = add_task(hi, json!({}), &TaskOptions::default());
    let d = add_task(hi2, json!({}), &TaskOptions::default());
    let e = add_task(hi2, json!({}), &TaskOptions::default());
    let f = add_task(hi2, json!({}), &TaskOptions::default());

    let _ = b >> (d | e | c) >> a >> f;

    let _ = expand(
        hi,
        &[Value::Null, Value::Null, Value::Null],
        &TaskOptions::default(),
    );
}
