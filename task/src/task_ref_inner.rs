use serde::Serialize;
use serde_json::json;
use std::{collections::HashSet, marker::PhantomData};

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
            "upstream_task_id": self.task_ids.iter().next().unwrap_or(&0),
        });

        if self.key.is_some() {
            json_value["key"] = serde_json::Value::String(self.key.clone().unwrap());
        }

        json_value.serialize(serializer)
    }
}
