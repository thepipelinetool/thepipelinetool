use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TriggerRule {
    AllSuccess,
    AnySuccess,

    AllDone,
    AnyDone,

    AnyFailed,
    AllFailed,
    // NONE_FAILED_OR_SKIPPED,
}
