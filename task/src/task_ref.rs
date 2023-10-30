use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{collections::HashSet, marker::PhantomData};

#[derive(Clone)]
pub struct TaskRef<T: Serialize> {
    pub task_ids: HashSet<usize>,
    pub _marker: PhantomData<T>,
}

impl<T: Serialize> Serialize for TaskRef<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let json_value = json!({
            "upstream_task_id": self.task_ids.iter().next().unwrap_or(&0),
        });

        json_value.serialize(serializer)
    }
}

impl<T: Serialize + DeserializeOwned> TaskRef<T> {
    pub fn get(&self, key: &str) -> Value {
        assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");
        json!({
            "upstream_task_id": self.task_ids.iter().next().unwrap(),
            "key": key.to_string(),
        })
    }
}

impl<T: Serialize> TaskRef<T> {
    pub fn value(&self) -> Value {
        assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");

        json!({
            "upstream_task_id": self.task_ids.iter().next().unwrap(),
        })
    }
}
