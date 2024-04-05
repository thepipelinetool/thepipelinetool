use std::time::Duration;

use chrono::TimeZone;
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DagOptions {
    #[serde(default)]
    pub schedule: Option<String>,

    #[serde(default)]
    pub start_date: Option<NaiveDateTime>,

    #[serde(default)]
    pub end_date: Option<NaiveDateTime>,

    #[serde(default)]
    pub max_attempts: usize,

    #[serde(default)]
    pub retry_delay: Duration,

    #[serde(default)]
    pub timeout: Option<Duration>,

    #[serde(default)]
    pub should_catchup: bool,

    #[serde(default)]
    pub timezone: Option<Tz>,
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
            should_catchup: false,
            timezone: None,
        }
    }
}

impl DagOptions {
    pub fn get_start_date_with_timezone(&self) -> Option<DateTime<Utc>> {
        naive_datetime_to_datetime_with_timezone(&self.start_date, &self.timezone)
    }

    pub fn get_end_date_with_timezone(&self) -> Option<DateTime<Utc>> {
        naive_datetime_to_datetime_with_timezone(&self.start_date, &self.timezone)
    }
}

fn naive_datetime_to_datetime_with_timezone(
    date: &Option<NaiveDateTime>,
    timezone: &Option<Tz>,
) -> Option<DateTime<Utc>> {
    if let Some(date) = date {
        if let Some(timezone) = timezone {
            return Some(
                timezone
                    .from_local_datetime(date)
                    .unwrap()
                    .with_timezone(&Utc),
            );
        }
        return Some(date.and_utc());
    }
    None
}
