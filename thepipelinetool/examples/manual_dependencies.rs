use thepipelinetool::prelude::*;

fn produce_data(_: ()) -> String {
    "world".to_string()
}

fn print_data(arg: String) -> () {
    println!("hello {arg}");
}

#[dag]
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
