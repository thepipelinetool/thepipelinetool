use thepipelinetool_core::{prelude::*, tpt};

fn hi(args: Value) -> Value {
    println!("{}", args);

    for i in 0..10 {
        // sleep(Duration::from_secs(3));
        println!("hello {i}");
    }

    json!({ "hello": "world" })
}

#[tpt::main]
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
