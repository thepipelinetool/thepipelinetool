use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct TaskOptions {
    pub max_attempts: usize,
    pub retry_delay: Duration,
    pub timeout: Option<Duration>,
    pub is_sensor: bool,
}

impl Default for TaskOptions {
    fn default() -> Self {
        Self {
            is_sensor: false,
            retry_delay: Duration::ZERO,
            timeout: None,
            max_attempts: 1,
        }
    }
}
