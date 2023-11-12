use std::vec;

use thepipelinetool::prelude::*;

fn main() {
    let a = add_command(
        json!(["bash", "-c", "sleep 2 && echo hello"]),
        &TaskOptions::default(),
    );
    let b = add_command(json!(["echo", a.value()]), &TaskOptions::default());

    let _c = vec![a, b];

    parse_cli();
}
