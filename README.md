# thepipelinetool

`thepipelinetool` is an *experimental* pipeline orchestration tool drawing on concepts from Apache Airflow.
It organizes Rust functions into a Directed Acyclic Graph (DAG) structure, ensuring orderly execution according to their dependencies.
The DAG is compiled into a CLI executable, which can then be used to list tasks/edges, run individual functions, and execute locally.
Finally, deploy to `thepipelinetool_server` to enjoy scheduling, catchup, and live task monitoring with a modern UI.

### Features
- *Safety and Reliability* - Rust's compile-time checks ensure code safety and prevent common bugs.
- *Scalable* - Designed to handle large-scale data processing tasks with ease.
- *Extensible* - Supports custom tasks and integrations, allowing for flexible workflow design.

### Deployment (WIP)
- Coming soon.

### Simple DAG
```rust
// simple.rs
use thepipelinetool::prelude::*;

// function argument type must derive Serialize & Deserialize
#[derive(Deserialize, Serialize)]
struct MyConfig {
    data: String,
}

fn produce_data(config: MyConfig) -> String {
    config.data
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[dag]
fn main() {
    // define your tasks and dependencies within the main() function

    // add a task that uses the function 'produce_data'
    let task_ref = add_task(produce_data, MyConfig{ data: "world".into() }, &TaskOptions::default());

    // add a task that will wait to use the result from produce_data
    let _ = add_task_with_ref(print_data, &task_ref, &TaskOptions::default());
}
```

Run using the following command
```bash
cargo run --bin simple run in_memory
```

Or use "help" to see all commands
```bash
cargo run --bin simple --help
```

### Manually Defining Depencencies
```rust
// chain.rs
use thepipelinetool::prelude::*;

fn produce_data(_: ()) -> String {
    "world".to_string()
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[dag]
fn main() {
    let task_ref = add_task(produce_data, (), &TaskOptions::default());
    
    let print_task1 = add_task_with_ref(print_data, &task_ref, &TaskOptions::default());
    let print_task2 = add_task_with_ref(print_data, &task_ref, &TaskOptions::default());
    // without further code, both tasks will run at the same time

    // Sequential:
    // UPSTREAM_TASK >> DOWNSTREAM_TASK
    // or DOWNSTREAM_TASK << UPSTREAM_TASK
    let _ = print_task1 >> print_task2; // print_task2 will now wait for print_task1 completion

    // Multiple Sequential:
    // task1 >> task2 >> task3;
    // task1 >> (task2 << task3) >> task4;

    // Parallel:
    // task1 >> (task2 | task3 | task4) >> task5;
}
```

### Dynamic Tasks
```rust
// dynamic.rs

use thepipelinetool::prelude::*;

fn produce_lazy(_: ()) -> Vec<u8> {
    vec![0, 1]
}

fn say_hello(arg: u8) -> u8 {
    println!("hello {arg}");
    arg
}

#[dag]
fn main() {
    let produce_lazy_task_ref = add_task(produce_lazy, (), &TaskOptions::default());

    // creates a new task for each item in "produce_lazy" result
    let expanded_lazy_task_ref = expand_lazy(say_hello, &produce_lazy_task_ref, &TaskOptions::default());

    // you can also chain lazily expanded tasks
    let _ = expand_lazy(say_hello, &expanded_lazy_task_ref, &TaskOptions::default());
}
```

### Branching Tasks
```rust
// branch.rs
use thepipelinetool::prelude::*;

fn branch_task(_: ()) -> Branch<usize> {
    Branch::Left(0)
}

fn left(arg: usize) -> () {
    println!("left {arg}");
}

fn right(_: usize) -> () {
    println!("this won't execute");
}

#[dag]
fn main() {
    // only "left" task will be executed since branch_task returns Branch::Left
    let _ = branch(branch_task, (), left, right, &TaskOptions::default());
}
```