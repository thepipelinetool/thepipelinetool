use thepipelinetool::prelude::*;

fn hi(args: Value) -> Value {
    println!("{}", args);

    json!({ "hello": "world" })
}

#[dag]
fn main() {
    let a = add_task(hi, json!({}), &TaskOptions::default());
    let b = add_task(hi, json!({}), &TaskOptions::default());
    let _c = add_task(
        hi,
        json!([a.value(), b.get("hello")]),
        &TaskOptions {
            timeout: None,
            ..Default::default()
        },
    );
}
