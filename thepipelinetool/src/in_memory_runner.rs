use std::{sync::mpsc::channel, thread};

use thepipelinetool_runner::{
    backend::Backend, blanket_backend::BlanketBackend, in_memory_backend::InMemoryBackend,
};

pub fn run_in_memory(
    backend: &mut InMemoryBackend,
    max_parallelism: usize,
    pipeline_source: &str,
    tpt_path: String,
) {
    let (tx, rx) = channel();
    let mut current_parallel_tasks_count = 0;

    for _ in 0..max_parallelism {
        if let Some(ordered_queued_task) = backend.pop_priority_queue().unwrap() {
            let tx = tx.clone();
            let mut backend = backend.clone();
            let (pipeline_source, tpt_path) =
                (pipeline_source.to_string(), tpt_path.clone());

            thread::spawn(move || {
                backend
                    .work(
                        ordered_queued_task.queued_task.run_id,
                        &ordered_queued_task,
                        &pipeline_source,
                        tpt_path,
                        ordered_queued_task.queued_task.scheduled_date_for_run,
                    )
                    .unwrap();
                backend
                    .remove_from_temp_queue(&ordered_queued_task.queued_task)
                    .unwrap();
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

        if let Some(ordered_queued_task) = backend.pop_priority_queue().unwrap() {
            let tx = tx.clone();
            let mut backend = backend.clone();
            let (pipeline_source, tpt_path) =
                (pipeline_source.to_string(), tpt_path.clone());

            thread::spawn(move || {
                backend
                    .work(
                        ordered_queued_task.queued_task.run_id,
                        &ordered_queued_task,
                        &pipeline_source,
                        tpt_path,
                        ordered_queued_task.queued_task.scheduled_date_for_run,
                    )
                    .unwrap();
                backend
                    .remove_from_temp_queue(&ordered_queued_task.queued_task)
                    .unwrap();
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
