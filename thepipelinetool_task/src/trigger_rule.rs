use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub enum TriggerRule {
    #[default]
    AllDone,
    AnyDone,

    AllSuccess,
    AnySuccess,

    AnyFailed,
    AllFailed,
}
