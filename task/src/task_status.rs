use serde::Deserialize;

#[derive(PartialEq, Clone, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Retrying,
    Success,
    Failure,
    Skipped,
}

impl TaskStatus {
    // Convert a TaskStatus to a &str
    pub fn as_str(&self) -> &'static str {
        match *self {
            TaskStatus::Pending => "Pending",
            TaskStatus::Running => "Running",
            TaskStatus::Retrying => "Retrying",
            TaskStatus::Success => "Success",
            TaskStatus::Failure => "Failure",
            TaskStatus::Skipped => "Skipped",
        }
    }

    pub fn as_u8(&self) -> u8 {
        match *self {
            TaskStatus::Pending => 0,
            TaskStatus::Running => 1,
            TaskStatus::Retrying => 2,
            TaskStatus::Success => 3,
            TaskStatus::Failure => 4,
            TaskStatus::Skipped => 5,
        }
    }

    // Convert a &str to a TaskStatus
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Pending" => Some(TaskStatus::Pending),
            "Running" => Some(TaskStatus::Running),
            "Retrying" => Some(TaskStatus::Retrying),
            "Success" => Some(TaskStatus::Success),
            "Failure" => Some(TaskStatus::Failure),
            "Skipped" => Some(TaskStatus::Skipped),
            _ => None,
        }
    }
}
