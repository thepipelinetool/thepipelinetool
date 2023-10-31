use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashSet, marker::PhantomData};

#[derive(Clone)]
pub struct TaskRef<T: Serialize> {
    pub task_ids: HashSet<usize>,
    pub key: Option<String>,
    pub _marker: PhantomData<T>,
}

impl<T: Serialize> Serialize for TaskRef<T> {
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

impl<T: Serialize> TaskRef<T> {
    pub fn get(&self, key: &str) -> TaskRef<Value> {
        assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");

        TaskRef {
            task_ids: self.task_ids.clone(),
            key: Some(key.to_string()),
            _marker: PhantomData,
        }
    }

    pub fn value(&self) -> TaskRef<Value> {
        assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");

        TaskRef {
            task_ids: self.task_ids.clone(),
            key: None,
            _marker: PhantomData,
        }
    }
}

// impl<T: Serialize + DeserializeOwned> TaskRef<T> {
//     pub fn get(&self, key: &str) -> Value {
//         assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");
//         json!({
//             "upstream_task_id": self.task_ids.iter().next().unwrap(),
//             "key": key.to_string(),
//         })
//     }
// }

// impl<T: Serialize> TaskRef<T> {
//     pub fn value(&self) -> Value {
//         assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");

//         json!({
//             "upstream_task_id": self.task_ids.iter().next().unwrap(),
//         })
//     }
// }

//     pub fn as_value(&self) -> TaskRef<Value> {
//         TaskRef {
//             task_ids: self.task_ids.clone(),
//             _marker: PhantomData,
//         }
//     }
// }
