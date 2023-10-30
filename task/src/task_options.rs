use std::time::Duration;

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TaskOptions {
    pub max_attempts: isize,
    pub retry_delay: Duration,
    pub timeout: Option<Duration>,
}

impl Default for TaskOptions {
    fn default() -> Self {
        Self {
            max_attempts: 1,
            retry_delay: Duration::ZERO,
            timeout: None,
        }
    }
}
