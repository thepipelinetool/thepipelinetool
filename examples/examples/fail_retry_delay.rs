use std::{time::Duration, vec};

use thepipelinetool_core::{prelude::*, tpt};

#[tpt::main]
fn main() {
    let a = add_task(
        bash_operator,
        json!(["bash", "-c", "sleep 3 && echo hello"]),
        &TaskOptions {
            timeout: Some(Duration::new(1, 0)),
            retry_delay: Duration::new(3, 0),
            max_attempts: 2,
            ..Default::default()
        },
    );
    let b = add_task(
        bash_operator,
        json!(["echo", a.value()]),
        &TaskOptions::default(),
    );

    let _c = vec![a, b];
}
