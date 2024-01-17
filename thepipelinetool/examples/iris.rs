use polars::prelude::*;
use thepipelinetool::prelude::*;

fn produce_data(_: ()) -> String {
    let q = LazyCsvReader::new("./thepipelinetool/examples/iris.csv")
        .has_header(true)
        .finish().unwrap()
        .filter(col("SepalLength").gt(lit(5)))
        .group_by(vec![col("Name")])
        .agg([col("*").sum()]);

    let df = q.collect().unwrap();

    dbg!(df);

    "world".to_string()
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[dag]
fn main() {
    let opts = &TaskOptions::default();

    // add a task that uses the function 'produce_data'
    let task_ref = add_task(produce_data, (), opts);

    // add a task that depends on 'task_ref'
    let _ = add_task_with_ref(print_data, &task_ref, opts);
}
