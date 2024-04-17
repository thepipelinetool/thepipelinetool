use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub enum TriggerRule {
    AllSuccess,
    AnySuccess,

    #[default]
    AllDone,
    AnyDone,

    AnyFailed,
    AllFailed,
    // NONE_FAILED_OR_SKIPPED,
}
