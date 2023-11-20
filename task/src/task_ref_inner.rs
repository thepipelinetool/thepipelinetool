use serde::Serialize;
use serde_json::json;
use std::{collections::HashSet, marker::PhantomData};

pub const UPSTREAM_TASK_ID_KEY: &str = "upstream_task_id";
pub const UPSTREAM_TASK_RESULT_KEY: &str = "key";

#[derive(Clone)]
pub struct TaskRefInner<T: Serialize> {
    pub task_ids: HashSet<usize>,
    pub key: Option<String>,
    pub _marker: PhantomData<T>,
}

impl<T: Serialize> Serialize for TaskRefInner<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut json_value = json!({
            UPSTREAM_TASK_ID_KEY: self.task_ids.iter().next().unwrap(),
        });

        if self.key.is_some() {
            json_value[UPSTREAM_TASK_RESULT_KEY] = serde_json::Value::String(self.key.clone().unwrap());
        }

        json_value.serialize(serializer)
    }
}
