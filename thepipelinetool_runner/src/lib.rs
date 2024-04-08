use std::{env, process::Command, sync::mpsc::channel, thread};

use backend::Backend;
use blanket_backend::BlanketBackend;

use serde::{Deserialize, Serialize};
use thepipelinetool_task::ordered_queued_task::OrderedQueuedTask;
use thepipelinetool_utils::spawn;

pub mod backend;
pub mod blanket_backend;
pub mod in_memory_backend;
pub mod options;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Executor {
    InMemory,
    Local,
    Docker,
    Kubernetes,
}

fn _run<U: Backend + BlanketBackend>(
    ordered_queued_task: &OrderedQueuedTask,
    mut backend: U,
    dag_path: Option<String>,
    tpt_path: Option<String>,
    executor: Executor,
) {
    match executor {
        Executor::InMemory => {
            backend.work(
                ordered_queued_task.queued_task.run_id,
                ordered_queued_task,
                dag_path.unwrap(), // TODO
                tpt_path.unwrap(),
                ordered_queued_task.queued_task.scheduled_date_for_dag_run,
            );
        }
        Executor::Local => {
            let mut cmd = Command::new(get_tpt_executor_command());
            cmd.arg(serde_json::to_string(ordered_queued_task).unwrap());
            let _ = spawn(
                cmd,
                Box::new(|x| print!("{x}")),
                Box::new(|x| eprint!("{x}")),
            );
        }
        Executor::Docker => {
            todo!()
        }
        Executor::Kubernetes => {
            todo!()
        }
    }
}

const DEFAULT_TPT_X_COMMAND: &str = "tpt_executor";

pub fn get_tpt_executor_command() -> String {
    env::var("TPT_X_CMD")
        .unwrap_or(DEFAULT_TPT_X_COMMAND.to_string())
        .to_string()
}

// fn pop_priority_queue(backend: &mut dyn Backend) -> Option<OrderedQueuedTask> {
//     backend.pop_priority_queue()
// }
// pub trait Runner<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> {
//     fn run(&mut self, ordered_queued_task: &OrderedQueuedTask);
//     fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask>;
// }

pub fn run<U: Backend + BlanketBackend + Send + Sync + Clone + 'static>(
    backend: &mut U,
    max_parallelism: usize,
    dag_path: Option<String>,
    tpt_path: Option<String>,
    executor: Executor,
) {
    let (tx, rx) = channel();
    let mut current_parallel_tasks_count = 0;

    for _ in 0..max_parallelism {
        if let Some(ordered_queued_task) = backend.pop_priority_queue() {
            let tx = tx.clone();
            let backend = backend.clone();
            let (dag_path, tpt_path) = (dag_path.clone(), tpt_path.clone());

            thread::spawn(move || {
                _run(&ordered_queued_task, backend, dag_path, tpt_path, executor);
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

        if let Some(ordered_queued_task) = backend.pop_priority_queue() {
            let tx = tx.clone();
            let backend = backend.clone();
            let (dag_path, tpt_path) = (dag_path.clone(), tpt_path.clone());

            thread::spawn(move || {
                _run(&ordered_queued_task, backend, dag_path, tpt_path, executor);
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
