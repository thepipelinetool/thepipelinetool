pub mod bash;

pub use bash::bash_operator;
use serde::{Deserialize, Serialize};
// use papermill::

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Operator {
    Bash,
}

impl Default for Operator {
    fn default() -> Self {
        Operator::Bash
    }
}