use std::{time::Duration, vec};

use thepipelinetool::prelude::*;

#[dag]
fn main() {
    let a = add_task(
        run_command,
        json!(["bash", "-c", "sleep 3 && echo hello"]),
        &TaskOptions {
            timeout: Some(Duration::new(1, 0)),
            retry_delay: Duration::new(3, 0),
            max_attempts: 2,
            ..Default::default()
        },
    );
    let b = add_task(
        run_command,
        json!(["echo", a.value()]),
        &TaskOptions::default(),
    );

    let _c = vec![a, b];
}
