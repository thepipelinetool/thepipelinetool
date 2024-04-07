use std::{sync::mpsc::channel, thread};

use backend::Backend;
use blanket_backend::BlanketBackend;

use thepipelinetool_task::ordered_queued_task::OrderedQueuedTask;

pub mod backend;
pub mod blanket_backend;
pub mod in_memory_backend;
pub mod in_memory_runner;
pub mod options;

pub trait Runner<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> {
    fn run(&mut self, ordered_queued_task: &OrderedQueuedTask);
    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask>;
}

pub fn run<U: Backend + BlanketBackend + Send + Sync + Clone + 'static>(
    runner: &mut (impl Runner<U> + Clone + Send + 'static),
    max_parallelism: usize,
) {
    let (tx, rx) = channel();
    let mut current_parallel_tasks_count = 0;

    for _ in 0..max_parallelism {
        if let Some(ordered_queued_task) = runner.pop_priority_queue() {
            let tx = tx.clone();
            let mut runner = runner.clone();

            thread::spawn(move || {
                runner.run(&ordered_queued_task);
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

    for _ in rx.iter() {
        current_parallel_tasks_count -= 1;

        if let Some(ordered_queued_task) = runner.pop_priority_queue() {
            let tx = tx.clone();
            let mut runner = runner.clone();

            thread::spawn(move || {
                runner.run(&ordered_queued_task);
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
