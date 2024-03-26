// // testing this use-case

// use serde::{de::DeserializeOwned, Serialize};
// use serde_json::Value;
// use thepipelinetool_task::{
//     branch::Branch, task_options::TaskOptions, task_ref_inner::TaskRefInner,
// };

// use crate::{functions::*, TaskRef};

// pub struct TaskBuilder {
//     options: TaskOptions,
// }

// impl TaskBuilder {
//     pub fn new() -> Self {
//         Self {
//             options: TaskOptions::default(),
//         }
//     }

//     pub fn options(mut self, options: &TaskOptions) -> Self {
//         self.options = *options;
//         self
//     }

//     pub fn build_expand<F, T, G, const N: usize>(
//         self,
//         function: F,
//         template_args_vec: &[T; N],
//     ) -> [TaskRef<TaskRefInner<G>>; N]
//     where
//         T: Serialize + DeserializeOwned,
//         G: Serialize,
//         F: Fn(T) -> G + 'static + Sync + Send,
//     {
//         expand(function, template_args_vec, &self.options)
//     }

//     pub fn build_with_ref<F, T, G>(self, function: F, task_ref: &TaskRef<T>) -> TaskRef<G>
//     where
//         T: Serialize + DeserializeOwned,
//         G: Serialize,
//         F: Fn(T) -> G + 'static + Sync + Send,
//     {
//         add_task_with_ref(function, task_ref, &self.options)
//     }

//     pub fn build<F, T, G>(self, function: F, template_args: T) -> TaskRef<G>
//     where
//         T: Serialize + DeserializeOwned,
//         G: Serialize,
//         F: Fn(T) -> G + 'static + Sync + Send,
//     {
//         add_task(function, template_args, &self.options)
//     }

//     pub fn build_branch<F, K, T, L, J, R, M>(
//         self,
//         function: F,
//         template_args: K,
//         left: L,
//         right: R,
//     ) -> (TaskRef<J>, TaskRef<M>)
//     where
//         T: Serialize + DeserializeOwned,
//         K: Serialize + DeserializeOwned,
//         J: Serialize,
//         M: Serialize,
//         F: Fn(K) -> Branch<T> + 'static + Sync + Send,
//         L: Fn(T) -> J + 'static + Sync + Send,
//         R: Fn(T) -> M + 'static + Sync + Send,
//     {
//         branch(function, template_args, left, right, &self.options)
//     }

//     pub fn build_lazy_expand<K, F, T, G>(
//         self,
//         function: F,
//         task_ref: &TaskRef<T>,
//     ) -> TaskRef<Vec<G>>
//     where
//         K: Serialize + DeserializeOwned,
//         T: Serialize + DeserializeOwned + IntoIterator<Item = K>,
//         G: Serialize,
//         F: Fn(K) -> G + 'static + Sync + Send,
//     {
//         expand_lazy(function, task_ref, &self.options)
//     }

//     pub fn build_bash_command(self, args: Value) -> TaskRef<Value> {
//         add_command(args, &self.options)
//     }
// }
