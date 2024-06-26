use thepipelinetool_core::{prelude::*, tpt};

fn hi(args: Value) -> Value {
    println!("{}", args);

    json!({ "hello": "world" })
}

#[tpt::main]
fn main() {
    // set_schedule("*/1 * * * *");

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
