use serde_json::{json, Value};

pub fn assert_operator(value: Value) -> Value {
    json!(value.is_boolean() && value.as_bool().expect(""))
}