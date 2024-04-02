use std::collections::HashSet;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
// use thepipelinetool_task::{
//     branch::Branch, task_options::TaskOptions, task_ref_inner::TaskRefInner, Task,
// };
// use thepipelinetool_utils::{collector, function_name_as_string};

use crate::{
    // prelude::*,
    dev::*,
    statics::{get_functions, get_tasks},
};

/// Expands a template using a provided function and template arguments.
///
/// This function takes a generic closure `function`, an array of template arguments
/// `template_args_vec`, and a reference to `TaskOptions`. It expands the template
/// for each template argument using the provided closure and returns an array of
/// `TaskRef<TaskRefInner<G>>` containing the expanded tasks.
///
/// # Type Parameters
///
/// - `F`: The type of the closure function.
/// - `T`: The type of the template arguments. Must be serializable and deserializable.
/// - `G`: The type of the output of the closure function. Must be serializable.
///
/// # Arguments
///
/// * `function` - A closure that takes a value of type `T` and returns a value of type `G`.
/// * `template_args_vec` - Arguments to the closure, which must implement `Serialize` and `DeserializeOwned`.
/// * `options` - A reference to the `TaskOptions` struct, containing configuration options for the task.
///
/// # Returns
///
/// An array of `TaskRef<TaskRefInner<G>>` with a length of `N`, where each element
/// represents an expanded task.
///
/// # Examples
///
/// ```rust
/// use thepipelinetool::prelude::*;
///
/// // Define a function to be used for expansion.
/// fn square(x: i32) -> i32 {
///     x * x
/// }
///
/// fn main() {
///     // Define an array of template arguments.
///     let template_args: [i32; 3] = [1, 2, 3];
///
///     // Expand the template using the `square` function and template arguments.
///     let expanded_tasks = expand(square, &template_args, &TaskOptions::default());
/// }
/// ```
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

    let wrapped_function = wrap_function(function);
    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name.clone(), Box::new(wrapped_function));
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

/// Adds a new task to the task management system with a reference to an existing task.
///
/// This function takes a generic closure `function`, a reference to an existing task `task_ref`,
/// and task options. It registers the new task in the system and returns a reference to the
/// newly added task.
///
/// # Type Parameters
///
/// - `F`: The type of the closure function.
/// - `T`: The type of the input task referenced by `task_ref`. Must be serializable and deserializable.
/// - `G`: The type of the output of the closure function. Must be serializable.
///
/// # Arguments
///
/// * `function` - A closure that takes a value of type `T` and returns a value of type `G`.
/// * `task_ref` - A reference to an existing task that provides the template arguments.
/// * `options` - A reference to the `TaskOptions` struct, containing configuration options for the task.
///
/// # Returns
///
/// Returns `TaskRef<G>`, a reference to the created task.
///
/// # Example
///
/// ```rust
/// use thepipelinetool::prelude::*;
///
/// // Define a function to be used for expansion.
/// fn double(x: i32) -> i32 {
///     x * 2
/// }
///
/// #[dag]
/// fn main() {
///     // Create an initial task with template arguments.
///     let initial_task_args = 5;
///     let initial_task = add_task(double, initial_task_args, &TaskOptions::default());
///
///     // Define an array of template arguments for the new task.
///     let new_task_args = 10;
///
///     // Add a new task based on the `double` function and reference the initial task.
///     let new_task_ref = add_task_with_ref(double, &initial_task, &TaskOptions::default());
/// }
/// ```
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
    let id = get_tasks().read().unwrap().len();

    let function_name = function_name_as_string(&function).to_string();
    {
        get_tasks().write().unwrap().insert(
            id,
            Task {
                id,
                name: function_name.to_string(),
                function: function_name.to_string(),
                template_args: serde_json::to_value(task_ref).unwrap(),
                options: *options,
                lazy_expand: false,
                is_dynamic: false,
                is_branch: false,
            },
        );
    }

    let wrapped_function = wrap_function(function);
    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name, Box::new(wrapped_function));
    }

    task_ref
        >> TaskRef(TaskRefInner {
            task_ids: HashSet::from([id]),
            key: None,

            _marker: std::marker::PhantomData,
        })
}

/// Adds a new task to the task management system.
///
/// This function takes a closure `function`, its template arguments `template_args`, and task options. It registers the task in the system and returns a reference to the newly added task.
///
/// # Type Parameters
///
/// - `F`: The type of the closure function.
/// - `T`: The type of the template arguments. Must be serializable and deserializable.
/// - `G`: The type of the output of the closure function. Must be serializable.
///
/// # Arguments
///
/// * `function` - A closure that takes a value of type `T` and returns a value of type `G`.
/// * `template_args` - Arguments to the closure, which must implement `Serialize` and `DeserializeOwned`.
/// * `options` - A reference to the `TaskOptions` struct, containing configuration options for the task.
///
/// # Returns
///
/// Returns `TaskRef<G>`, a reference to the created task.
pub fn add_task<F, T, G>(function: F, template_args: T, options: &TaskOptions) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let name = &function_name_as_string(&function).to_string();
    _add_task(function, template_args, options, name)
}

pub fn _add_task<F, T, G>(
    function: F,
    template_args: T,
    options: &TaskOptions,
    name: &str,
) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned + 'static,
    G: Serialize + 'static,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let id = get_tasks().read().unwrap().len();

    let function_name = function_name_as_string(&function).to_string();
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

    let wrapped_function = wrap_function(function);
    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name.to_string(), Box::new(wrapped_function));
    }
    TaskRef(TaskRefInner {
        task_ids: HashSet::from([id]),
        key: None,

        _marker: std::marker::PhantomData,
    })
}

/// Creates a branching task with two possible outcomes.
///
/// This function registers a new branching task based on a specified function and its template arguments. It returns two `TaskRef` instances representing the two possible branches of execution.
///
/// # Type Parameters
///
/// - `F`: The type of the branching function.
/// - `K`: The type of the template arguments for the branching function.
/// - `T`: The intermediate type produced by the branching function.
/// - `L`: The type of the left branch function.
/// - `J`: The output type of the left branch.
/// - `R`: The type of the right branch function.
/// - `M`: The output type of the right branch.
///
/// # Arguments
///
/// * `function` - A branching function that takes a value of type `K` and returns a `Branch<T>`.
/// * `template_args` - Arguments for the branching function, which must implement `Serialize` and `DeserializeOwned`.
/// * `left` - A function to be executed if the left branch is taken.
/// * `right` - A function to be executed if the right branch is taken.
/// * `options` - A reference to `TaskOptions` struct, containing configuration options for the task.
///
/// # Returns
///
/// Returns a tuple of two `TaskRef` instances, one for each branch (`left` and `right`).
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

    let wrapped_function = wrap_function(function);
    // move |value: Value| -> Value {
    //     // Deserialize the Value into T
    //     let input: K = serde_json::from_value(value).unwrap();
    //     // Call the original function
    //     let output: Branch<T> = function(input);
    //     // Serialize the G type back into Value
    //     serde_json::to_value(output).unwrap()
    // };
    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name, Box::new(wrapped_function));
    }
    let b = TaskRef::<T>(TaskRefInner::<T> {
        task_ids: HashSet::from([id]),
        key: None,
        _marker: std::marker::PhantomData,
    });

    (
        add_task_with_ref(left, &b, options),
        add_task_with_ref(right, &b, options),
    )
}

/// Dynamically expands a collection of tasks based on a transformation function.
///
/// This function takes a task and applies a transformation to each item in the task's collection,
/// returning a new task reference with the transformed items. It's designed to work with tasks
/// that require serialization and deserialization of their input and output.
///
/// # Type Parameters
///
/// - `K`: The type of items in the input task's collection. Must be serializable and deserializable.
/// - `T`: The type of the task's collection. Must be serializable, deserializable, and convertible
///   into an iterator over items of type `K`.
/// - `G`: The type of items in the output task's collection. Must be serializable.
/// - `F`: The transformation function type. Takes an item of type `K` and transforms it into type `G`.
///
/// # Arguments
///
/// - `function`: A function that defines how each item of type `K` in the task's collection will
///   be transformed into type `G`. Must be thread-safe and static (`'static`), and implement both
///   `Sync` and `Send` traits.
/// - `task_ref`: A reference to the task containing the collection of items to be transformed.
/// - `options`: Configuration options for the task.
///
/// # Returns
///
/// Returns a `TaskRef<Vec<G>>`, which is a reference to the new task containing the transformed
/// items.
///
/// # Examples
///
/// ```
/// use thepipelinetool::prelude::*;
///
/// fn lazy_producer(_: ()) -> Vec<usize> {
///     vec![0, 1, 2]
/// }
/// #[dag]
/// fn main() {
///     let lazy_task = add_task(lazy_producer, (), &TaskOptions::default());
///     expand_lazy(|i: usize| println!("{i}"), &lazy_task, &TaskOptions::default());
/// }
/// ```
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
    let id = get_tasks().read().unwrap().len();

    let function_name = function_name_as_string(&function).to_string();
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

    let wrapped_function = wrap_function(function);
    // move |value: Value| -> Value {
    //     // Deserialize the Value into T
    //     let input: K = serde_json::from_value(value).unwrap();
    //     // Call the original function
    //     let output: G = function(input);
    //     // Serialize the G type back into Value
    //     serde_json::to_value(output).unwrap()
    // };
    {
        let mut functions = get_functions().write().unwrap();
        functions.insert(function_name.to_string(), Box::new(wrapped_function));
        functions.insert(function_name_as_string(collector), Box::new(collector));
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
        // Deserialize the Value into T
        let input: K = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: T = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    }
}
