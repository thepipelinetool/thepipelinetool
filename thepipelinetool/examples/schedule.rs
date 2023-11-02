use chrono::DateTime;
use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: Value) -> Value {
        println!("{}", args);

        json!({
            "hello": "world"
        })
    }

    let mut dag = DAG::new();
    dag.set_schedule("0 0 12 * *");
    dag.set_start_date(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());

    let a = dag.add_task(hi, json!({}), TaskOptions::default());
    let b = dag.add_task(hi, json!({}), TaskOptions::default());
    let _c = dag.add_task(hi, json!([a.value(), b.get("hello")]), TaskOptions{
        timeout: None,
        ..Default::default()
    });

    dag.parse_cli();
}
