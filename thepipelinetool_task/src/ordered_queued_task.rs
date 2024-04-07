use serde::{Deserialize, Serialize};

use crate::queued_task::QueuedTask;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrderedQueuedTask {
    pub score: usize,
    pub queued_task: QueuedTask,
}

impl Ord for OrderedQueuedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .score
            .cmp(&self.score)
            .then_with(|| other.queued_task.task_id.cmp(&self.queued_task.task_id))
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::max_by(self, other, Ord::cmp)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::min_by(self, other, Ord::cmp)
    }
}

impl PartialOrd for OrderedQueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
