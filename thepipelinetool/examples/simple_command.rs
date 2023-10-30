use std::vec;

use thepipelinetool::prelude::*;

fn main() {
    let mut dag = DAG::new("simple_command");
    let a = dag.add_command(json!(["bash", "-c", "sleep 2 && echo hello"]), TaskOptions::default());
    let b = dag.add_command(json!(["echo", a.value()]), TaskOptions::default());

    let _c = vec![a, b];

    dag.apply_cli_args();
}
