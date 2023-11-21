use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct QueuedTask {
    pub task_id: usize,
    pub run_id: usize,
    pub dag_name: String,
    pub queued_date: DateTime<FixedOffset>,
    pub attempt: usize,
}
