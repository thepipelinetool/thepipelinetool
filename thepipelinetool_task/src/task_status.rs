use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    RetryPending,
    Success,
    Failure,
    Skipped,
}

impl TaskStatus {
    pub fn as_u8(&self) -> u8 {
        match *self {
            TaskStatus::Pending => 0,
            TaskStatus::Running => 1,
            TaskStatus::RetryPending => 2,
            TaskStatus::Success => 3,
            TaskStatus::Failure => 4,
            TaskStatus::Skipped => 5,
        }
    }
}
