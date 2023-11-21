use std::vec;

use thepipelinetool::prelude::*;

#[dag]
fn main() {
    let a = add_command(
        json!(["bash", "-c", "sleep 2 && echo hello"]),
        &TaskOptions::default(),
    );
    let _ = add_command(json!(["echo", a.value()]), &TaskOptions::default());
}
