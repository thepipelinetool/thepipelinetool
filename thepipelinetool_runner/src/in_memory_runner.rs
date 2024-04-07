use std::path::PathBuf;

use thepipelinetool_task::ordered_queued_task::OrderedQueuedTask;

use crate::{backend::Backend, blanket_backend::BlanketBackend, Runner};

#[derive(Clone)]
pub struct InMemoryRunner<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> {
    pub backend: Box<U>,
    pub tpt_path: String,
    pub dag_path: PathBuf,
}

impl<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> Runner<U> for InMemoryRunner<U> {
    fn run(&mut self, ordered_queued_task: &OrderedQueuedTask) {
        self.backend.work(
            ordered_queued_task.queued_task.run_id,
            ordered_queued_task,
            self.dag_path.clone(),
            self.tpt_path.clone(),
            ordered_queued_task.queued_task.scheduled_date_for_dag_run,
        );
    }

    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask> {
        self.backend.pop_priority_queue()
    }
}
