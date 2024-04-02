use std::collections::HashSet;

use serde::de::DeserializeOwned;

use crate::dev::*;

pub fn _add_task_with_function_name<T, G>(
    template_args: Value,
    options: &TaskOptions,
    name: &str,
    function_name: &str,
) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
{
    let id = get_tasks().read().unwrap().len();

    {
        get_tasks().write().unwrap().insert(
            id,
            Task {
                id,
                name: name.to_string(),
                function: function_name.to_string(),
                template_args: serde_json::to_value(template_args).unwrap(),
                options: *options,
                lazy_expand: false,
                is_dynamic: false,
                is_branch: false,
            },
        );
    }
    TaskRef(TaskRefInner {
        task_ids: HashSet::from([id]),
        key: None,

        _marker: std::marker::PhantomData,
    })
}

pub fn _register_function_with_name<G, T, F>(function: F, name: &str)
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    get_functions()
        .write()
        .unwrap()
        .insert(name.to_string(), Box::new(_wrap_function(function)));
}

pub fn _expand_lazy_with_function_name<K, T, G>(
    task_ref: &TaskRef<T>,
    options: &TaskOptions,
    name: &str,
    function_name: &str,
) -> TaskRef<Vec<G>>
where
    K: Serialize + DeserializeOwned + 'static,
    T: Serialize + DeserializeOwned + IntoIterator<Item = K>,
    G: Serialize + 'static,
{
    let id = get_tasks().read().unwrap().len();

    {
        get_tasks().write().unwrap().insert(
            id,
            Task {
                id,
                name: name.to_string(),
                function: function_name.to_string(),
                template_args: serde_json::to_value(task_ref).unwrap(),
                options: *options,
                lazy_expand: true,
                is_dynamic: false,
                is_branch: false,
            },
        );
    }

    task_ref
        >> TaskRef(TaskRefInner {
            task_ids: HashSet::from([id]),
            key: None,
            _marker: std::marker::PhantomData,
        })
}

pub fn _wrap_function<K, T, F>(function: F) -> impl Fn(Value) -> Value
where
    T: Serialize,
    K: Serialize + DeserializeOwned,
    F: Fn(K) -> T + 'static + Sync + Send,
{
    move |value: Value| -> Value {
        let input: K = serde_json::from_value(value).unwrap();
        let output: T = function(input);
        serde_json::to_value(output).unwrap()
    }
}
