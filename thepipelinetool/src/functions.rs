use std::collections::HashSet;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::{
    dev::*,
    statics::{get_functions, get_tasks},
};

pub fn expand<F, T, G, const N: usize>(
    function: F,
    template_args_vec: &[T; N],
    options: &TaskOptions,
) -> [TaskRef<TaskRefInner<G>>; N]
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let function_name = function_name_as_string(&function).to_string();
    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name.clone(), Box::new(wrap_function(function)));
    }

    let mut i = 0;

    [(); N].map(|_| {
        let id = get_tasks().read().unwrap().len();
        {
            get_tasks().write().unwrap().insert(
                id,
                Task {
                    id,
                    name: function_name.to_string(),
                    function: function_name.clone(),
                    template_args: serde_json::to_value(&template_args_vec[i]).unwrap(),
                    options: *options,
                    lazy_expand: false,
                    is_dynamic: false,
                    is_branch: false,
                },
            );
        }
        i += 1;

        TaskRef(TaskRefInner {
            task_ids: HashSet::from([id]),
            key: None,
            _marker: std::marker::PhantomData,
        })
    })
}

pub fn add_task_with_ref<F, T, G>(
    function: F,
    task_ref: &TaskRef<T>,
    options: &TaskOptions,
) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let name = function_name_as_string(&function).to_string();
    task_ref
        >> _add_task(
            function,
            serde_json::to_value(task_ref).unwrap(),
            options,
            &name,
        )
}

pub fn add_task<F, T, G>(function: F, template_args: T, options: &TaskOptions) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let name = &function_name_as_string(&function).to_string();
    _add_task(
        function,
        serde_json::to_value(template_args).unwrap(),
        options,
        name,
    )
}

pub fn _add_task<F, T, G>(
    function: F,
    template_args: Value,
    options: &TaskOptions,
    name: &str,
) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let function_name = function_name_as_string(&function).to_string();

    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name.to_string(), Box::new(wrap_function(function)));
    }

    _add_task_with_function_name::<T, G>(template_args, options, name, &function_name)
}

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

pub fn branch<F, K, T, L, J, R, M>(
    function: F,
    template_args: K,
    left: L,
    right: R,
    options: &TaskOptions,
) -> (TaskRef<J>, TaskRef<M>)
where
    T: Serialize + DeserializeOwned + 'static,
    K: Serialize + DeserializeOwned + 'static,
    J: Serialize + 'static,
    M: Serialize + 'static,
    F: Fn(K) -> Branch<T> + 'static + Sync + Send,
    L: Fn(T) -> J + 'static + Sync + Send,
    R: Fn(T) -> M + 'static + Sync + Send,
{
    let id = get_tasks().read().unwrap().len();
    let function_name = function_name_as_string(&function).to_string();

    {
        get_tasks().write().unwrap().insert(
            id,
            Task {
                id,
                name: function_name.to_string(),
                function: function_name.to_string(),
                template_args: serde_json::to_value(template_args).unwrap(),
                options: *options,
                lazy_expand: false,
                is_dynamic: false,
                is_branch: true,
            },
        );
    }

    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name, Box::new(wrap_function(function)));
    }
    let task_ref = TaskRef::<T>(TaskRefInner::<T> {
        task_ids: HashSet::from([id]),
        key: None,
        _marker: std::marker::PhantomData,
    });

    (
        add_task_with_ref(left, &task_ref, options),
        add_task_with_ref(right, &task_ref, options),
    )
}

pub fn expand_lazy<K, F, T, G>(
    function: F,
    task_ref: &TaskRef<T>,
    options: &TaskOptions,
) -> TaskRef<Vec<G>>
where
    K: Serialize + DeserializeOwned + 'static,
    T: Serialize + DeserializeOwned + IntoIterator<Item = K>,
    G: Serialize + 'static,
    F: Fn(K) -> G + 'static + Sync + Send,
{
    let name = &function_name_as_string(&function).to_string();
    _expand_lazy(function, task_ref, options, name)
}

pub fn _expand_lazy<K, F, T, G>(
    function: F,
    task_ref: &TaskRef<T>,
    options: &TaskOptions,
    name: &str,
) -> TaskRef<Vec<G>>
where
    K: Serialize + DeserializeOwned + 'static,
    T: Serialize + DeserializeOwned + IntoIterator<Item = K>,
    G: Serialize + 'static,
    F: Fn(K) -> G + 'static + Sync + Send,
{
    let function_name = function_name_as_string(&function).to_string();
    {
        let mut functions = get_functions().write().unwrap();
        functions.insert(function_name.to_string(), Box::new(wrap_function(function)));
        functions.insert(function_name_as_string(collector), Box::new(collector));
    }

    _expand_lazy_with_function_name::<K, T, G>(task_ref, options, name, &function_name)
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

pub fn wrap_function<K, T, F>(function: F) -> impl Fn(Value) -> Value
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
