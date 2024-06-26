use std::{sync::mpsc::channel, thread};

use thepipelinetool_runner::{
    backend::Backend, blanket_backend::BlanketBackend, in_memory_backend::InMemoryBackend,
};

pub fn run_in_memory(backend: &mut InMemoryBackend, max_parallelism: usize, tpt_path: String) {
    let (tx, rx) = channel();
    let mut current_parallel_tasks_count = 0;

    for _ in 0..max_parallelism {
        if let Some(temp_queued_task) = backend.pop_priority_queue().unwrap() {
            let tx = tx.clone();
            let mut backend = backend.clone();
            let tpt_path = tpt_path.clone();

            thread::spawn(move || {
                backend.work(&temp_queued_task, tpt_path).unwrap();
                backend.remove_from_temp_queue(&temp_queued_task).unwrap();
                tx.send(()).unwrap();
            });

            current_parallel_tasks_count += 1;
            if current_parallel_tasks_count >= max_parallelism {
                break;
            }
        } else {
            break;
        }
    }

    if current_parallel_tasks_count == 0 {
        drop(tx);
        return;
    }

    for _ in rx.iter() {
        current_parallel_tasks_count -= 1;

        if let Some(temp_queued_task) = backend.pop_priority_queue().unwrap() {
            let tx = tx.clone();
            let mut backend = backend.clone();
            let tpt_path = tpt_path.clone();

            thread::spawn(move || {
                backend.work(&temp_queued_task, tpt_path).unwrap();
                backend.remove_from_temp_queue(&temp_queued_task).unwrap();
                tx.send(()).unwrap();
            });
            current_parallel_tasks_count += 1;

            if current_parallel_tasks_count >= max_parallelism {
                continue;
            }
        }
        if current_parallel_tasks_count == 0 {
            drop(tx);
            break;
        }
    }
}
