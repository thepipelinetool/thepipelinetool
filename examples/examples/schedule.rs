// use chrono::DateTime;
use thepipelinetool_core::{prelude::*, tpt};

fn hi(args: Value) -> Value {
    println!("{}", args);

    json!({ "hello": "world" })
}

#[tpt::main]
fn main() {
    // set_schedule("0 0 12 * *");
    // set_start_date(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());
    // set_end_date(DateTime::parse_from_rfc3339("1997-06-19T16:39:57-08:00").unwrap());
    // set_catchup(true);

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
