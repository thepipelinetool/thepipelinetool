use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use std::{collections::HashSet, marker::PhantomData};
use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

#[derive(Clone)]
pub struct TaskRefInner<T: Serialize> {
    pub task_ids: HashSet<usize>,
    pub key: Option<String>,
    pub _marker: PhantomData<T>,
}

impl<T: Serialize> Serialize for TaskRefInner<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut json_value = json!({
            UPSTREAM_TASK_ID_KEY: self.task_ids.iter().next().unwrap(),
        });

        if self.key.is_some() {
            json_value[UPSTREAM_TASK_RESULT_KEY] = Value::String(self.key.clone().unwrap());
        }

        json_value.serialize(serializer)
    }
}
