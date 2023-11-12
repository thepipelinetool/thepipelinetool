use thepipelinetool::prelude::*;

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

#[dag]
fn main() {
    let a = add_task(hi, json!({}), &TaskOptions::default());
    let _ = add_task_with_ref(hi, &a, &TaskOptions::default());
    let _ = add_task_with_ref(hi, &a.get("hello"), &TaskOptions::default());
    let b = add_task_with_ref(hi2, &a, &TaskOptions::default());
    let _ = add_task_with_ref(hi2, &b, &TaskOptions::default());
}
