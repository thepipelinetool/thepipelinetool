use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Run {
    pub run_id: usize,
    pub pipeline_name: String,
    pub scheduled_date_for_run: DateTime<Utc>,
}

impl Run {
    pub fn dummy() -> Self {
        Self {
            run_id: 0,
            pipeline_name: "dummy".to_string(),
            scheduled_date_for_run: Utc::now(),
        }
    }
}
