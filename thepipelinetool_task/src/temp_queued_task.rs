use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::queued_task::QueuedTask;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Hash, Clone)]
pub struct TempQueuedTask {
    pub popped_date: DateTime<Utc>,
    pub queued_task: QueuedTask,
}
