pub mod assert;
pub mod bash;
pub mod params;
pub mod print;

pub use bash::bash_operator;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    #[default]
    BashOperator,
    ParamsOperator,
    PrintOperator,
    AssertOperator,
}

pub const ORIGINAL_STRING_KEY: &str = "_original_string";

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
