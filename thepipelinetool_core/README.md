```rust
use thepipelinetool_core::{prelude::*, tpt};

#[tpt::main]
fn main() {
    // define your tasks here
}
```

## Documentation
- Find the latest docs [here](https://docs.rs/thepipelinetool/latest/thepipelinetool/)

## Examples
### Simple DAG
```rust
use thepipelinetool_core::{prelude::*, tpt};

fn produce_data(_: ()) -> String {
    "world".to_string()
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[tpt::main]
fn main() {
    let opts = &TaskOptions::default();

    // add a task that uses the function 'produce_data'
    let task_ref = add_task(produce_data, (), opts);

    // add a task that depends on 'produce_data'
    let _ = add_task_with_ref(print_data, &task_ref, opts);
}
```

```mermaid
flowchart TD
  id0(produce_data_0)
  style id0 color:black,stroke:grey,fill:white,stroke-width:4px
  id1(print_data_1)
  style id1 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id1
```
----
### Manually Defining Depencencies
```rust
use thepipelinetool_core::{prelude::*, tpt};

fn produce_data(_: ()) -> String {
    "world".to_string()
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[tpt::main]
fn main() {
    let opts = &TaskOptions::default();
    let task_ref = add_task(produce_data, (), opts);

    // these tasks will execute in parallel
    let _task_ref1 = add_task_with_ref(print_data, &task_ref, opts);
    let _task_ref2 = add_task_with_ref(print_data, &task_ref, opts);

    // declare downstream dependencies using right-shift operator '>>'
    let task_ref3 = add_task_with_ref(print_data, &task_ref, opts);
    let task_ref4 = add_task_with_ref(print_data, &task_ref, opts);
    let _ = task_ref4 >> task_ref3; // run task4 before task3

    // declare upstream dependencies using left-shift operator '<<'
    let task_ref5 = add_task_with_ref(print_data, &task_ref, opts);
    let task_ref6 = add_task_with_ref(print_data, &task_ref, opts);
    let _ = &task_ref5 << task_ref6; // run task6 before task5

    // declare parallel tasks using bitwise-or operator '|'
    let task_ref7 = add_task_with_ref(print_data, &task_ref, opts);
    let task_ref8 = add_task_with_ref(print_data, &task_ref, opts);
    let parallel_task_ref = task_ref7 | task_ref8; // run task7 and task8 in parallel

    // use previous results for further dependency declaration
    let _ = parallel_task_ref >> task_ref5;

    // chaining
    let task_ref9 = add_task_with_ref(print_data, &task_ref, opts);
    let task_ref10 = add_task_with_ref(print_data, &task_ref, opts);
    let task_ref11 = add_task_with_ref(print_data, &task_ref, opts);

    let _ = task_ref9 >> task_ref10 >> task_ref11;
    // the result of taskA >> taskB is taskB, so the above is equivalent to:
    // ((task_ref9 >> task_ref10) >> task_ref11)
}
```
```mermaid
flowchart TD
  id0(produce_data_0)
  style id0 color:black,stroke:grey,fill:white,stroke-width:4px
  id1(print_data_1)
  style id1 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id1
  id2(print_data_2)
  style id2 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id2
  id3(print_data_3)
  style id3 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id3
  id4-->id3
  id4(print_data_4)
  style id4 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id4
  id5(print_data_5)
  style id5 color:black,stroke:grey,fill:white,stroke-width:4px
  id8-->id5
  id7-->id5
  id0-->id5
  id6-->id5
  id6(print_data_6)
  style id6 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id6
  id7(print_data_7)
  style id7 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id7
  id8(print_data_8)
  style id8 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id8
  id9(print_data_9)
  style id9 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id9
  id10(print_data_10)
  style id10 color:black,stroke:grey,fill:white,stroke-width:4px
  id9-->id10
  id0-->id10
  id11(print_data_11)
  style id11 color:black,stroke:grey,fill:white,stroke-width:4px
  id10-->id11
  id0-->id11
```
----
### Branching Tasks
```rust
use thepipelinetool_core::{prelude::*, tpt};

fn branch_task(_: ()) -> Branch<usize> {
    Branch::Left(0)
}

fn left(arg: usize) -> () {
    println!("left {arg}");
}

fn right(_: usize) -> () {
    println!("this won't execute");
}

#[tpt::main]
fn main() {
    // only 'left' task will be executed since branch_task returns Branch::Left
    let _ = branch(branch_task, (), left, right, &TaskOptions::default());
}
```
```mermaid
flowchart TD
  id0(branch_task_0)
  style id0 color:black,stroke:grey,fill:white,stroke-width:4px
  id1(left_1)
  style id1 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id1
  id2(right_2)
  style id2 color:black,stroke:grey,fill:white,stroke-width:4px
  id0-->id2
```
----
### Dynamic Tasks
```rust
use thepipelinetool_core::{prelude::*, tpt};

fn produce_lazy(_: ()) -> Vec<u8> {
    vec![0, 1]
}

fn say_hello(arg: u8) -> u8 {
    println!("hello {arg}");
    arg
}

#[tpt::main]
fn main() {
    let opts = &TaskOptions::default();

    let produce_lazy_task_ref = add_task(produce_lazy, (), opts);

    // creates a new task for each item in 'produce_lazy' result
    let expanded_lazy_task_ref = expand_lazy(say_hello, &produce_lazy_task_ref, opts);

    // you can also chain lazily expanded tasks
    let _ = expand_lazy(say_hello, &expanded_lazy_task_ref, opts);
}
```
----
### More Examples
Find more examples [here](https://github.com/thepipelinetool/thepipelinetool/tree/main/thepipelinetool/examples)

## Deployment
To deploy pipelines, the compiled binaries must be placed inside `PIPELINES_DIR` for both the server and workers to access.
Visit the [template](https://github.com/thepipelinetool/thepipelinetool_template) project for the docker-compose.yml example

## License
AGPLv3
