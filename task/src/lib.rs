use serde::Serialize;
use serde_json::Value;

pub mod task;
pub mod task_options;
pub mod task_ref;
pub mod task_result;
pub mod task_status;

#[derive(Serialize)]
pub struct Branch<T: Serialize> {
    is_left: bool,
    val: T,
}

impl<T: Serialize> Branch<T> {
    pub fn left(val: T) -> Self {
        Self { is_left: true, val }
    }
    
    pub fn right(val: T) -> Self {
        Self { is_left: true, val }
    }

    pub fn get_val(&self) -> Value {
        serde_json::to_value(&self.val).unwrap()
    }
}
