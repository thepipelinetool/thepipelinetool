use std::process::Command;

use serde::{Deserialize, Serialize};
use thepipelinetool_core::dev::OrderedQueuedTask;
use thepipelinetool_runner::{backend::Backend, blanket_backend::BlanketBackend, Runner};
use thepipelinetool_utils::spawn;

use crate::env::{get_executor_type, get_tpt_executor_command};

#[derive(Clone, Serialize, Deserialize)]
pub enum Executor {
    Local,
    Docker,
    Kubernetes,
}

#[derive(Clone)]
pub struct LocalRunner<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> {
    pub backend: Box<U>,
    executor: Executor,
}

impl<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> LocalRunner<U> {
    pub fn new(backend: U) -> Self {
        Self {
            backend: Box::new(backend),
            executor: get_executor_type(),
        }
    }
}

impl<U: Backend + BlanketBackend + Send + Sync + Clone + 'static> Runner<U> for LocalRunner<U> {
    fn run(&mut self, ordered_queued_task: &OrderedQueuedTask) {
        match self.executor {
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

    fn pop_priority_queue(&mut self) -> Option<OrderedQueuedTask> {
        self.backend.pop_priority_queue()
    }
}
