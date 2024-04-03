pub mod bash;
pub mod params;
pub mod print;

pub use bash::bash_operator;
use serde::{Deserialize, Serialize};
// use papermill::

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    #[default]
    BashOperator,
    ParamsOperator,
    PrintOperator,
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use crate::Operator;

    #[test]
    fn test() {
        assert_eq!(
            serde_json::from_str::<Value>(&serde_json::to_string(&Operator::BashOperator).unwrap())
                .unwrap(),
            "bash_operator"
        );
    }
}

// impl Default for Operator {
//     fn default() -> Self {
//         Operator::Bash
//     }
// }
