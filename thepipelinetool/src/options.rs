use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DagOptions {
    pub schedule: Option<String>,
    pub start_date: Option<DateTime<FixedOffset>>,
    pub end_date: Option<DateTime<FixedOffset>>,
    pub max_attempts: usize,
    pub retry_delay: Duration,
    pub timeout: Option<Duration>,
    pub catchup: bool,
}

impl Default for DagOptions {
    fn default() -> Self {
        Self {
            schedule: None,
            start_date: None,
            end_date: None,
            max_attempts: 1,
            retry_delay: Duration::ZERO,
            timeout: None,
            catchup: false,
        }
    }
}
