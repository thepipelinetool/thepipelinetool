use std::str::FromStr;

#[derive(PartialEq, Clone, Debug)]
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
}

impl FromStr for TaskStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(TaskStatus::Pending),
            "Running" => Ok(TaskStatus::Running),
            "Retrying" => Ok(TaskStatus::Retrying),
            "Success" => Ok(TaskStatus::Success),
            "Failure" => Ok(TaskStatus::Failure),
            "Skipped" => Ok(TaskStatus::Skipped),
            _ => Err(()),
        }
    }
}
