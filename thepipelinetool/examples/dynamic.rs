use thepipelinetool::prelude::*;

fn main() {
    fn hi(args: u8) -> u8 {
        println!("{}", args);
        args
    }

    fn hi2(args: Value) -> [u8; 2] {
        println!("{}", args);

        [0, 1]
    }

    let a = add_task(hi2, json!({}), TaskOptions::default());
    let b = expand_lazy(hi, &a, TaskOptions::default());
    // let c = add_task(hi2, json!({}), TaskOptions::default());

    // seq(&[&b, &c]);

    expand_lazy(hi, &b, TaskOptions::default());

    parse_cli();
    // println!("{}", get_mermaid_graph());
}
