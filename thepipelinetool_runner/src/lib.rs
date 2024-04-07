use std::{
    sync::mpsc::{channel, Sender},
    thread::{self, JoinHandle},
};

use backend::Backend;
use blanket_backend::BlanketBackend;

use thepipelinetool_task::ordered_queued_task::{self, OrderedQueuedTask};

pub mod backend;
pub mod blanket_backend;
pub mod in_memory;
pub mod options;

pub trait Runner<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> {
    fn work(&mut self, ordered_queued_task: &OrderedQueuedTask);

    fn get_max_parallelism(&self) -> usize;
    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask>;
}

pub fn run<U: Backend + BlanketBackend + Send + Sync + Clone + 'static>(
    runner: &mut (impl Runner<U> + Clone + Send + 'static),
    // spawner: K,
) 
// where
    // K: Fn(Box<dyn FnMut() + Send + 'static>) -> JoinHandle<T>, // T: Send + 'static,
{
    // let mut backend = runner.backend.clone();
    let max_parallelism = runner.get_max_parallelism();

    let (tx, rx) = channel();
    let mut thread_count = 0;

    for _ in 0..max_parallelism {
        if let Some(ordered_queued_task) = runner.pop_priority_queue() {
            let tx = tx.clone();
            let mut runner = runner.clone();
            // let k = move || {
            //     runner.work(&ordered_queued_task);
            //     tx.send(()).unwrap();
            // };
            thread::spawn(Box::new(move || {
                runner.work(&ordered_queued_task);
                tx.send(()).unwrap();
            }));

            thread_count += 1;
            if thread_count >= max_parallelism {
                break;
            }
        } else {
            break;
        }
    }

    for _ in rx.iter() {
        thread_count -= 1;

        if let Some(ordered_queued_task) = runner.pop_priority_queue() {
            let tx = tx.clone();
            let mut runner = runner.clone();
            // thread::spawn(move || {
            //     runner.work(&ordered_queued_task);
            //     tx.send(()).unwrap();
            // });
            thread::spawn(Box::new(move || {
                runner.work(&ordered_queued_task);
                tx.send(()).unwrap();
            }));
            thread_count += 1;

            if thread_count >= max_parallelism {
                continue;
            }
        }
        if thread_count == 0 {
            drop(tx);
            break;
        }
    }
}

// fn spawn<U: Backend + BlanketBackend + Send + Sync + Clone + 'static>(
//     tx: Sender<()>,
//     runner: &mut (impl Runner<U> + Clone + Send + 'static),
//     ordered_queued_task: &OrderedQueuedTask,
// ) {
//     runner.work(&ordered_queued_task);
//     tx.send(()).unwrap();
// }

// fn _run<F: Fn() + Send + 'static>(f: F) {
//     thread::spawn(f);
// }
