use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DagOptions {
    #[serde(default)]
    pub schedule: Option<String>,

    #[serde(default)]
    pub start_date: Option<DateTime<FixedOffset>>,

    #[serde(default)]
    pub end_date: Option<DateTime<FixedOffset>>,

    #[serde(default)]
    pub max_attempts: usize,

    #[serde(default)]
    pub retry_delay: Duration,

    #[serde(default)]
    pub timeout: Option<Duration>,

    #[serde(default)]
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
