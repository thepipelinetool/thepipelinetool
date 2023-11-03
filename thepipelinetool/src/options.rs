use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct DagOptions<'a> {
    pub schedule: Option<&'a str>,
    pub start_date: Option<DateTime<FixedOffset>>,
    pub end_date: Option<DateTime<FixedOffset>>,
    pub max_attempts: usize,
    pub retry_delay: Duration,
    pub timeout: Option<Duration>,
    pub catchup: bool,
}

impl<'a> DagOptions<'a> {
    pub fn set_schedule(&mut self, schedule: &'a str) {
        self.schedule = Some(schedule);
    }

    pub fn set_start_date(&mut self, start_date: DateTime<FixedOffset>) {
        self.start_date = Some(start_date);
    }

    pub fn set_end_date(&mut self, end_date: DateTime<FixedOffset>) {
        self.end_date = Some(end_date);
    }
}

impl<'a> Default for DagOptions<'a> {
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