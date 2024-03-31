pub mod bash;

pub use bash::bash_operator;
use serde::{Deserialize, Serialize};
// use papermill::

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum Operator {
    #[default]
    Bash,
}

// impl Default for Operator {
//     fn default() -> Self {
//         Operator::Bash
//     }
// }
