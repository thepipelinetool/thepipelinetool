use serde::{Deserialize, Serialize};
use thepipelinetool::prelude::*;

#[derive(Serialize, Deserialize)]
struct Test {
    val: String,
}

#[derive(Serialize)]
struct TestResult {
    res: String,
}

fn main() {
    fn hi(args: Value) -> Value {
        println!("{}", args);

        json!({
            "hello": "world"
        })
    }

    fn hi2(args: Test) -> TestResult {
        println!("{}", args.val);

        TestResult {
            res: "world".into(),
        }
    }

    fn hi3(args: Value) -> Vec<Test> {
        println!("{}", args);

        vec![
            Test {
                val: "hell234o2!!".into(),
            },
            Test {
                val: "hello342!!".into(),
            },
        ]
    }

    fn hi4(args: Test) -> Value {
        println!("{}", args.val);

        json!({
            "hello": "world"
        })
    }

    let mut dag = DAG::new();

    let a = dag.add_task(
        hi2,
        Test {
            val: "hello!!".into(),
        },
        TaskOptions::default(),
    );
    let _ = dag.add_task_with_ref(hi, &a.value(), TaskOptions::default());
    let _ = dag.add_task_with_ref(hi, &a.get("res"), TaskOptions::default());

    let b = dag.add_task(hi, json!({}), TaskOptions::default());
    let _c = dag.add_task(
        hi,
        json!([a.value(), b.get("hello")]),
        TaskOptions {
            timeout: None,
            ..Default::default()
        },
    );

    dag.expand(
        hi2,
        &[
            Test {
                val: "hello!!".into(),
            },
            Test {
                val: "hello2!!".into(),
            },
        ],
        TaskOptions::default(),
    );

    let a = dag.add_task(hi3, json!({}), TaskOptions::default());
    let _h = dag.expand_lazy(hi4, &a, TaskOptions::default());

    dag.parse_cli();
}
