use polars::prelude::*;
use thepipelinetool_core::prelude::*;

fn produce_data(_: ()) -> Value {
    let q = LazyCsvReader::new("examples/iris.csv")
        .has_header(true)
        .finish()
        .unwrap()
        .filter(col("SepalLength").gt(lit(5)))
        .group_by(vec![col("Name")])
        .agg([col("*").sum()]);

    let df = q.collect().unwrap();

    dbg!(&df);

    serde_json::to_value(&df).unwrap()
}

fn print_data(arg: Value) -> () {
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

#[cfg(test)]
mod tests {
    use crate::produce_data;

    #[test]
    fn test_data() {
        dbg!(produce_data(()));
    }
}
