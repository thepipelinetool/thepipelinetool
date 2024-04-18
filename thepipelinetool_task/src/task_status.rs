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
