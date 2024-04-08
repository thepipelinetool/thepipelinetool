use std::collections::HashSet;

use serde::de::DeserializeOwned;

use crate::dev::*;

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
    let function_name = register_function(function);

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
                    use_trigger_params: false,
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
    let function_name = register_function(function);

    task_ref
        >> _add_task_with_function_name::<T, G>(
            serde_json::to_value(task_ref).unwrap(),
            options,
            &function_name,
            &function_name,
            false
        )
}

pub fn add_task<F, T, G>(function: F, template_args: T, options: &TaskOptions) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let function_name = register_function(function);

    _add_task_with_function_name::<T, G>(
        serde_json::to_value(template_args).unwrap(),
        options,
        &function_name,
        &function_name,
        false,
    )
}

pub fn add_task_using_trigger_params<F, T, G>(function: F, options: &TaskOptions) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let function_name = register_function(function);

    _add_task_with_function_name::<T, G>(
        Value::Null,
        options,
        &function_name,
        &function_name,
        true,
    )
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
    let function_name = register_function(function);

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
                use_trigger_params: false,
            },
        );
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
    register_function(collector);
    let function_name = register_function(function);

    _expand_lazy_with_function_name::<K, T, G>(task_ref, options, &function_name, &function_name)
}

pub fn register_function<G, T, F>(function: F) -> String
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let function_name = function_name_as_string(&function).to_string();
    _register_function_with_name(function, &function_name);
    function_name
}
