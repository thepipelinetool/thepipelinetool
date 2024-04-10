use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Hash, Clone)]
pub struct QueuedTask {
    pub task_id: usize,
    pub run_id: usize,
    pub pipeline_name: String,
    pub scheduled_date_for_run: DateTime<Utc>,
    pub attempt: usize,
}
