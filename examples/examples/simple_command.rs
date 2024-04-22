use std::vec;

use thepipelinetool_core::{prelude::*, tpt};

#[tpt::main]
fn main() {
    let a = add_task(
        bash_operator,
        json!(["bash", "-c", "sleep 2 && echo hello"]),
        &TaskOptions::default(),
    );
    let _ = add_task(
        bash_operator,
        json!(["echo", a.value()]),
        &TaskOptions::default(),
    );
}
