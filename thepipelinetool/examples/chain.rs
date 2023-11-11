use thepipelinetool::prelude::*;

fn main() {
    fn hi(_args: Value) -> Value {
        println!("hi");

        json!({
            "hello": "world"
        })
    }

    let a = add_task(hi, json!({}), TaskOptions::default());
    let b = add_task(hi, json!({}), TaskOptions::default());
    let c = add_task(hi, json!({}), TaskOptions::default());

    let d = add_task(hi, json!({}), TaskOptions::default());
    let e = add_task(hi, json!({}), TaskOptions::default());

    let p = &par(&d, &e);
    // seq(&[&c, &b, &p, &a]);
    let _ = a >> b >> c;

    let _out = expand(
        hi,
        &[Value::Null, Value::Null, Value::Null],
        TaskOptions::default(),
    );

    parse_cli();
    // println!("{}", get_initial_mermaid_graph());
}
