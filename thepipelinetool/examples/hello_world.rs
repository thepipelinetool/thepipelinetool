use std::{thread, time::Duration};

use rand::Rng;

use thepipelinetool::prelude::*;

fn hello_world0(args: Value) -> Value {
    println!("hello world0 {:?}", args);

    // let mut m = Map::new();
    // m.insert("data".to_string(), (1 + 3).into());
    // Some(m)
    thread::sleep(Duration::from_secs(3));
    json!({
        "data": (2 + 2)
    })
}

fn hello_world1(args: Value) -> Value {
    println!("hello world1 {:?}", args);
    // assert!(false);

    json!({
        "data": (5 + 2)
    })
}

fn hello_world2(args: Value) -> Value {
    println!("hello world2 {:?}", args);
    let mut rng = rand::thread_rng(); // Initialize the random number generator

    let random_bool: bool = rng.gen(); // Generate a random boolean value
    println!("{}", random_bool);
    thread::sleep(Duration::from_secs(3));

    json!({
        "data": (5 + 2)
    })
}

fn hello_world3(args: Value) -> Value {
    println!("hello world3 {:?}", args);
    thread::sleep(Duration::from_secs(3));

    assert!(false);

    json!({
        "data": (5 + 2)
    })
}

fn main() {
    // let mut dag = DAG::new(true);
    let mut dag = DAG::new("hello_world");
    // let mut dag = DAG::new_persistent("".into(), true, SqliteRunner::new());

    let task0 = dag.add_task(hello_world0, Value::Null, TaskOptions::default());

    let task1 = dag.add_task(hello_world2, Value::Null, TaskOptions::default());
    let a = dag.add_task(
        hello_world2,
        json!({
            "hi": task1.get("data"),
            "hello": task1.get("data"),
        }),
        TaskOptions::default(),
    );
    let b = dag.add_task(
        hello_world1,
        json!({
            "hi": task1.get("data"),
            "hello": task1.get("data"),
        }),
        TaskOptions::default(),
    );
    let d = dag.add_task(
        hello_world2,
        json!({
            "hi": a.get("data"),
            "hello": b.get("data"),
        }),
        TaskOptions::default(),
    );

    let task2 = dag.add_task(
        hello_world1,
        json!({"hi": task0.get("data")}),
        TaskOptions::default(),
    );

    let task3 = dag.add_task(
        hello_world3,
        json!({
            "hi": task0.get("data"),
            "hello": task2.get("data"),
        }),
        TaskOptions::default(),
    );

    let _task4 = dag.add_task(
        hello_world3,
        json!({
            "hi": task3.get("data"),
            "hello": task2.get("data"),
            "hey": task0.get("data"),
            "whatup": task0.value(),
            "howdy": "hello",
        }),
        TaskOptions::default(),
    );

    let anonymous = dag.add_task(
        |args| -> Value { args },
        json!({"data": "anonymous"}),
        TaskOptions::default(),
    );

    let _ = dag.add_task(
        hello_world1,
        json!({
            "hi": anonymous.get("data"),
            "hello": anonymous.get("data"),
        }),
        TaskOptions::default(),
    );
    let c = dag.add_task(
        hello_world1,
        json!({
            "hi": anonymous.get("data"),
            "hello": anonymous.get("data"),
        }),
        TaskOptions::default(),
    );

    let x = dag.add_task(hello_world1, Value::Null, TaskOptions::default());

    let _y = dag.add_task(
        hello_world1,
        json!({
            "hi": x.get("data"),
        }),
        TaskOptions::default(),
    );
    // task0 >> task2;
    dag.seq(&[&task1, &c]);
    dag.seq(&[&anonymous, &d]);
    // anonymous >> a;

    // a >> task0;
    // &y >> &x;
    // (task1, task2) >> (c, c);

    // dag.save_graph();
    dag.apply_cli_args();
    // println!("{}", dag.get_mermaid_graph());

    // function_name_as_string(|args| -> Value { args });
}
