use serde::Serialize;
use serde_json::json;
use std::{collections::HashSet, marker::PhantomData};

#[derive(Clone)]
pub struct TaskRefInner<T: Serialize> {
    pub task_ids: HashSet<usize>,
    pub key: Option<String>,
    pub _marker: PhantomData<T>,
}

// pub trait TaskRefTrait<T> {
//     fn get(&self, key: &str) -> TaskRef<Value>;
//     fn value(&self) -> TaskRef<Value>;
// }

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

// impl<T: Serialize> TaskRef<T> {
//     pub fn get(&self, key: &str) -> TaskRef<Value> {
//         assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");

//         TaskRef {
//             task_ids: self.task_ids.clone(),
//             key: Some(key.to_string()),
//             _marker: PhantomData,
//         }
//     }

//     pub fn value(&self) -> TaskRef<Value> {
//         assert!(self.task_ids.len() == 1, "Cannot use parallel ref as arg");

//         TaskRef {
//             task_ids: self.task_ids.clone(),
//             key: None,
//             _marker: PhantomData,
//         }
//     }
// }

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

// impl Shr for TaskRef {

// }

// impl<T, G> Shr<TaskRef<G>> for TaskRef<T>
// where
//     T: Serialize,
//     G: Serialize,
//     // R: From<(HashSet<usize>, Option<String>)>,
//     // Define any constraints necessary for T, G, and R
//     // For example, T might need to support a certain operation
//     // that is used in the implementation below.
// {
//     // type Output = R; // Define the output type of the operation
//     type Output = TaskRef<G>; // Define the output type of the operation

//     fn shr(self, rhs: TaskRef<G>) -> Self::Output {
//         // Define the behavior of the shift right operation
//         // This could involve manipulating `self.value` and `rhs.value`
//         // and producing a result of type R.
//         let dag = get_dag();
//     }
// }
