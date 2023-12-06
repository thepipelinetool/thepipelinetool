//! # thepipelinetool
//!
//! `thepipelinetool` organizes your Rust functions into a [Directed Acyclic Graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph) (DAG) structure, ensuring orderly execution according to their dependencies.
//! The DAG is compiled into a CLI executable, which can then be used to list tasks/edges, run individual functions, and execute locally. Finally, deploy to [thepipelinetool_server](https://github.com/thepipelinetool/thepipelinetool_server) to enjoy scheduling, catchup, retries, and live task monitoring with a modern web UI.

// mod options;

// mod module {
//     #[macro_export]
//     macro_rules! add_task {
//         ($mand_1:expr) => {
//             add_task!($mand_1, Value::Null)
//         };
//         ($mand_1:expr, $mand_2:expr) => {
//             add_task!($mand_1, $mand_2, &TaskOptions::default())
//         };
//         ($mand_1:expr, $mand_2:expr, $mand_3:expr) => {
//             add_task($mand_1, $mand_2, $mand_3)
//         };
//     }
//     pub use add_task;
// }

pub mod prelude {
    // pub use crate::module::*;
    // pub use crate::options::DagOptions;
    pub use crate::{
        add_command, add_task, add_task_with_ref, branch, expand, expand_lazy, parse_cli,
        // set_catchup, set_end_date, set_schedule, set_start_date,
    };
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use thepipelinetool_proc_macro::dag;
    pub use thepipelinetool_runner::in_memory::InMemoryRunner;
    pub use thepipelinetool_runner::{blanket::BlanketRunner, Runner};
    pub use thepipelinetool_task::branch::Branch;
    pub use thepipelinetool_task::ordered_queued_task::OrderedQueuedTask;
    pub use thepipelinetool_task::queued_task::QueuedTask;
    pub use thepipelinetool_task::task_options::TaskOptions;
    pub use thepipelinetool_task::task_result::TaskResult;
    pub use thepipelinetool_task::task_status::TaskStatus;
    pub use thepipelinetool_task::Task;
    pub use thepipelinetool_utils::execute_function_using_json_files;
}

// use chrono::{DateTime, FixedOffset};
// use options::DagOptions;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::ops::Shl;
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::{BitOr, Shr},
    process::Command,
};
use thepipelinetool_task::branch::Branch;
use thepipelinetool_task::Task;
use thepipelinetool_task::{task_options::TaskOptions, task_ref_inner::TaskRefInner};
use thepipelinetool_utils::{collector, execute_function_using_json_str_args, function_name_as_string};

type StaticTasks = RwLock<Vec<Task>>;
type StaticFunctions = RwLock<HashMap<String, Box<dyn Fn(Value) -> Value + Sync + Send>>>;
type StaticEdges = RwLock<HashSet<(usize, usize)>>;
// type StaticOptions = RwLock<DagOptions>;

use std::{
    cmp::max,
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
    sync::mpsc::channel,
    thread,
};

use chrono::Utc;
use clap::{arg, command, value_parser, Command as CliCommand};
// use saffron::{
//     parse::{CronExpr, English},
//     Cron,
// };
use thepipelinetool_runner::{blanket::BlanketRunner, in_memory::InMemoryRunner, Runner};
use thepipelinetool_utils::{execute_function_using_json_files, to_base62};

static TASKS: OnceLock<StaticTasks> = OnceLock::new();
static FUNCTIONS: OnceLock<StaticFunctions> = OnceLock::new();
static EDGES: OnceLock<StaticEdges> = OnceLock::new();
// static OPTIONS: OnceLock<StaticOptions> = OnceLock::new();

fn get_tasks() -> &'static StaticTasks {
    TASKS.get_or_init(StaticTasks::default)
}

fn get_functions() -> &'static StaticFunctions {
    FUNCTIONS.get_or_init(StaticFunctions::default)
}

fn get_edges() -> &'static StaticEdges {
    EDGES.get_or_init(StaticEdges::default)
}

// fn get_options() -> &'static StaticOptions {
//     OPTIONS.get_or_init(StaticOptions::default)
// }

impl<T, G> Shr<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: TaskRef<G>) -> Self::Output {
        seq(&self, &rhs)
    }
}

impl<T, G> Shr<&TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(self, rhs)
    }
}

impl<T, G> Shr<TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: TaskRef<G>) -> Self::Output {
        seq(self, &rhs)
    }
}

impl<T, G> Shr<&TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(&self, rhs)
    }
}

impl<T, G> Shl<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: TaskRef<G>) -> Self::Output {
        seq(&rhs, &self)
    }
}

impl<T, G> Shl<&TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(rhs, self)
    }
}

impl<T, G> Shl<TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: TaskRef<G>) -> Self::Output {
        seq(&rhs, self)
    }
}

impl<T, G> Shl<&TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(rhs, &self)
    }
}

impl<T, G> BitOr<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: TaskRef<G>) -> Self::Output {
        par(&self, &rhs)
    }
}

impl<T, G> BitOr<&TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: &TaskRef<G>) -> Self::Output {
        par(self, rhs)
    }
}

impl<T, G> BitOr<TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: TaskRef<G>) -> Self::Output {
        par(self, &rhs)
    }
}

impl<T, G> BitOr<&TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: &TaskRef<G>) -> Self::Output {
        par(&self, rhs)
    }
}

impl<T: Serialize> Serialize for TaskRef<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T: Serialize> TaskRef<T> {
    pub fn get(&self, key: &str) -> TaskRef<Value> {
        assert!(self.0.task_ids.len() == 1, "Cannot use parallel ref as arg");

        TaskRef(TaskRefInner {
            task_ids: self.0.task_ids.clone(),
            key: Some(key.to_string()),
            _marker: PhantomData,
        })
    }

    pub fn value(&self) -> TaskRef<Value> {
        assert!(self.0.task_ids.len() == 1, "Cannot use parallel ref as arg");

        TaskRef(TaskRefInner {
            task_ids: self.0.task_ids.clone(),
            key: None,
            _marker: PhantomData,
        })
    }
}
pub struct TaskRef<T: Serialize>(TaskRefInner<T>);

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
    T: Serialize + DeserializeOwned,
    G: Serialize,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let function_name = function_name_as_string(&function).to_string();

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: T = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };
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
                    function_name: function_name.clone(),
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
    T: Serialize + DeserializeOwned,
    G: Serialize,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let id = get_tasks().read().unwrap().len();

    let function_name = function_name_as_string(&function).to_string();
    {
        get_tasks().write().unwrap().insert(
            id,
            Task {
                id,
                function_name: function_name.to_string(),
                template_args: serde_json::to_value(task_ref).unwrap(),
                options: *options,
                lazy_expand: false,
                is_dynamic: false,
                is_branch: false,
            },
        );
    }

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: T = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };
    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name, Box::new(wrapped_function));
    }
    TaskRef(TaskRefInner {
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
///
/// # Examples
///
/// ```
/// use thepipelinetool::prelude::*;
///
/// #[dag]
/// fn main() {
///     let task_ref = add_task(
///         |arg: i32| arg + 1,     // function
///         5,                      // template_args
///         &TaskOptions::default() // options
///     );
/// }
/// ```
pub fn add_task<F, T, G>(function: F, template_args: T, options: &TaskOptions) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned,
    G: Serialize,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let id = get_tasks().read().unwrap().len();

    let function_name = function_name_as_string(&function).to_string();
    {
        get_tasks().write().unwrap().insert(
            id,
            Task {
                id,
                function_name: function_name.to_string(),
                template_args: serde_json::to_value(template_args).unwrap(),
                options: *options,
                lazy_expand: false,
                is_dynamic: false,
                is_branch: false,
            },
        );
    }

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: T = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };
    {
        get_functions()
            .write()
            .unwrap()
            .insert(function_name, Box::new(wrapped_function));
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
///
/// # Examples
///
/// ```
/// use thepipelinetool::prelude::*;
///
/// #[dag]
/// fn main() {
///     let (left_ref, right_ref) = branch(
///         |arg: i32| if arg > 10 { Branch::Left(arg) } else { Branch::Right(arg) },
///         5,
///         |left_arg| left_arg + 1,
///         |right_arg| right_arg - 1,
///         &TaskOptions::default()
///     );
/// }
/// ```
pub fn branch<F, K, T, L, J, R, M>(
    function: F,
    template_args: K,
    left: L,
    right: R,
    options: &TaskOptions,
) -> (TaskRef<J>, TaskRef<M>)
where
    T: Serialize + DeserializeOwned,
    K: Serialize + DeserializeOwned,
    J: Serialize,
    M: Serialize,
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
                function_name: function_name.to_string(),
                template_args: serde_json::to_value(template_args).unwrap(),
                options: *options,
                lazy_expand: false,
                is_dynamic: false,
                is_branch: true,
            },
        );
    }

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: K = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: Branch<T> = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };
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
    K: Serialize + DeserializeOwned,
    T: Serialize + DeserializeOwned + IntoIterator<Item = K>,
    G: Serialize,
    F: Fn(K) -> G + 'static + Sync + Send,
{
    let id = get_tasks().read().unwrap().len();

    let function_name = function_name_as_string(&function).to_string();
    {
        get_tasks().write().unwrap().insert(
            id,
            Task {
                id,
                function_name: function_name.to_string(),
                template_args: serde_json::to_value(task_ref).unwrap(),
                options: *options,
                lazy_expand: true,
                is_dynamic: false,
                is_branch: false,
            },
        );
    }

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: K = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };
    {
        let mut functions = get_functions().write().unwrap();
        functions.insert(function_name.clone(), Box::new(wrapped_function));
        functions.insert(function_name_as_string(collector), Box::new(collector));
    }

    TaskRef(TaskRefInner {
        task_ids: HashSet::from([id]),
        key: None,
        _marker: std::marker::PhantomData,
    })
}

/// Adds a command task to the task management system.
///
/// This function takes a JSON `args` value representing an array of command arguments
/// and a reference to `TaskOptions`. It registers the command task in the system and
/// returns a reference to the newly added task.
///
/// # Arguments
///
/// * `args` - A JSON `Value` representing an array of command arguments. The array
///   should contain the command and its arguments as strings.
/// * `options` - A reference to the `TaskOptions` struct, containing configuration
///   options for the task.
///
/// # Returns
///
/// Returns `TaskRef<Value>`, a reference to the created command task.
///
/// # Examples
///
/// ```rust
/// use thepipelinetool::prelude::*;
///
/// fn main() {
///     // Define command arguments as a JSON array.
///     let command_args: Value = json!(["ls", "-l"]);
//////
///     // Add a command task using the `add_command` function.
///     let command_task = add_command(command_args, &TaskOptions::default());
/// }
/// ```
pub fn add_command(args: Value, options: &TaskOptions) -> TaskRef<Value> {
    assert!(args.is_array());
    add_task(run_command, args, options)
}

fn seq<T: Serialize, G: Serialize>(a: &TaskRef<T>, b: &TaskRef<G>) -> TaskRef<G> {
    let mut last: usize = 0;
    let mut edges = get_edges().write().unwrap();

    for up in a.0.task_ids.iter() {
        for down in b.0.task_ids.iter() {
            edges.insert((*up, *down));
            last = *down;
        }
    }

    TaskRef(TaskRefInner {
        task_ids: HashSet::from([last]),
        key: None,
        _marker: std::marker::PhantomData,
    })
}

fn par<T: Serialize, G: Serialize>(a: &TaskRef<T>, b: &TaskRef<G>) -> TaskRef<G> {
    let mut task_ids: HashSet<usize> = HashSet::new();
    task_ids.extend(&a.0.task_ids);
    task_ids.extend(&b.0.task_ids);

    TaskRef(TaskRefInner {
        task_ids,
        key: None,
        _marker: std::marker::PhantomData,
    })
}

fn run_command(args: Value) -> Value {
    let mut args: Vec<&str> = args
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let output = Command::new(args[0])
        .args(&mut args[1..])
        .output()
        .unwrap_or_else(|_| panic!("failed to run command:\n{}\n\n", args.join(" ")));
    let result_raw = String::from_utf8_lossy(&output.stdout);
    let err_raw = String::from_utf8_lossy(&output.stderr);

    println!("{}", result_raw);
    if !output.status.success() {
        eprintln!("{}", err_raw);
        panic!("failed to run command:\n{}\n\n", args.join(" "));
    }

    json!(result_raw.to_string().trim_end())
}

// /// Sets the schedule for task execution in the task management system.
// ///
// /// This function takes a `schedule` string and updates the schedule option in the
// /// system's task options. The schedule determines when tasks are executed based
// /// on a defined pattern.
// ///
// /// # Arguments
// ///
// /// * `schedule` - A string representing the schedule pattern.
// ///
// /// # Example
// ///
// /// ```rust
// /// use thepipelinetool::prelude::*;
// ///
// /// #[dag]
// /// fn main() {
// ///     // Set a new schedule pattern.
// ///     set_schedule("0 0 * * *");
// ///
// ///     // ...
// /// }
// /// ```
// pub fn set_schedule(schedule: &str) {
//     get_options().write().unwrap().schedule = Some(schedule.to_string());
// }

// /// Sets the start date for task execution in the task management system.
// ///
// /// This function takes a `start_date` of type `DateTime<FixedOffset>` and updates
// /// the start date option in the system's task options. The start date specifies when
// /// task execution should begin.
// ///
// /// # Arguments
// ///
// /// * `start_date` - A `DateTime<FixedOffset>` representing the desired start date.
// ///
// /// # Example
// ///
// /// ```rust
// /// use thepipelinetool::prelude::*;
// ///
// /// #[dag]
// /// fn main() {
// ///     // Set a new schedule pattern.
// ///     set_start_date(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());
// ///
// ///     // ...
// /// }
// /// ```
// pub fn set_start_date(start_date: DateTime<FixedOffset>) {
//     get_options().write().unwrap().start_date = Some(start_date);
// }

// /// Sets the end date for task execution in the task management system.
// ///
// /// This function takes an `end_date` of type `DateTime<FixedOffset>` and updates
// /// the end date option in the system's task options. The end date specifies when
// /// task execution should stop.
// ///
// /// # Arguments
// ///
// /// * `end_date` - A `DateTime<FixedOffset>` representing the desired end date.
// ///
// /// # Example
// ///
// /// ```rust
// /// use thepipelinetool::prelude::*;
// ///
// /// #[dag]
// /// fn main() {
// ///     set_end_date(DateTime::parse_from_rfc3339("1997-06-19T16:39:57-08:00").unwrap());
// ///
// ///     // ...
// /// }
// /// ```
// pub fn set_end_date(end_date: DateTime<FixedOffset>) {
//     get_options().write().unwrap().end_date = Some(end_date);
// }

// /// Sets the catchup option for task execution in the task management system.
// ///
// /// This function takes a `catchup` boolean value and updates the catchup option
// /// in the system's task options. When catchup is enabled, the system will attempt
// /// to execute tasks that were missed during periods of inactivity.
// ///
// /// # Arguments
// ///
// /// * `catchup` - A boolean value indicating whether catchup should be enabled (`true`)
// ///   or disabled (`false`).
// ///
// /// # Example
// ///
// /// ```rust
// ///  use thepipelinetool::prelude::*;
// ///
// /// #[dag]
// /// fn main() {
// ///     set_catchup(true);
// ///
// ///     // ...
// /// }
// /// ```
// pub fn set_catchup(catchup: bool) {
//     get_options().write().unwrap().catchup = catchup;
// }

/// Parses command-line arguments and executes various tasks in the DAG CLI tool.
///
/// This function parses command-line arguments using the `command!` macro and executes
/// corresponding tasks based on the subcommands and options provided. It interacts with
/// the task management system to perform operations like displaying task information, running
/// tasks, and more.
///
/// The `parse_cli` function is typically called in the `main` function of your Rust application.
/// If you are using the #[dag] macro, it will automatically add a `parse_cli()` function call
/// to the end of the `main` function, simplifying the setup.
///
/// # Examples
///
///
/// ```rust
/// #[dag]
/// fn main() {
///     // your code here
///
///     // The #[dag] macro adds a parse_cli() function call to the end of the main function
/// }
/// ```
/// is equivalent to
/// ```rust
/// fn main() {
///     // your code here
///     parse_cli();
/// }
/// ```
///
/// The behavior of the CLI tool depends on the subcommands and options passed on the command
/// line. Use the "--help" command to see the CLI details.
pub fn parse_cli() {
    let command = command!()
        .about("DAG CLI Tool")
        .subcommand(CliCommand::new("describe").about("Describes the DAG"))
        // .subcommand(CliCommand::new("options").about("Displays options as JSON"))
        .subcommand(CliCommand::new("tasks").about("Displays tasks as JSON"))
        .subcommand(CliCommand::new("edges").about("Displays edges as JSON"))
        .subcommand(CliCommand::new("hash").about("Displays hash as JSON"))
        .subcommand(
            CliCommand::new("graph")
                .about("Displays graph")
                .arg_required_else_help(true)
                .arg(
                    arg!(
                        [graph_type] "Type of graph to output"
                    )
                    .required(true)
                    .value_parser(value_parser!(String))
                    .default_values(["mermaid", "graphite"])
                    .default_missing_value("mermaid"),
                ),
        )
        .subcommand(CliCommand::new("tree").about("Displays tree"))
        .subcommand(
            CliCommand::new("run")
                .about("Run complete DAG or function by name")
                .arg_required_else_help(true)
                .subcommand(
                    CliCommand::new("in_memory")
                        .about("Runs this DAG in memory")
                        .arg(
                            arg!(
                                [num_threads] "Max number of threads for parallel execution"
                            )
                            .required(false)
                            .value_parser(value_parser!(String))
                            .default_value("max")
                            .default_missing_value("max"),
                        ),
                )
                .subcommand(
                    CliCommand::new("function")
                        .about("Runs function")
                        .arg(
                            arg!(
                                <function_name> "Function name"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <in_path> "Input file"
                            )
                            .required(true),
                        )
                        .arg(
                            arg!(
                                <out_path> "Output file"
                            )
                            .required(false),
                        ),
                )
                .subcommand_required(true),
        )
        .subcommand_required(true);

    let matches = command.get_matches();

    if let Some(subcommand) = matches.subcommand_name() {
        match subcommand {
            // "options" => {
            //     let options = get_options().read().unwrap();

            //     println!(
            //         "{}",
            //         serde_json::to_string_pretty(&options.clone()).unwrap()
            //     );
            // }
            "describe" => {
                let tasks = get_tasks().read().unwrap();
                // let options = get_options().read().unwrap();
                let functions = get_functions().read().unwrap();

                println!("Task count: {}", tasks.len());
                println!(
                    "Functions: {:#?}",
                    functions.keys().collect::<Vec<&String>>()
                );

                // if let Some(schedule) = &options.schedule {
                //     println!("Schedule: {schedule}");
                //     match schedule.parse::<CronExpr>() {
                //         Ok(cron) => {
                //             println!("Description: {}", cron.describe(English::default()));
                //         }
                //         Err(err) => {
                //             println!("{err}: {schedule}");
                //             return;
                //         }
                //     }

                //     match schedule.parse::<Cron>() {
                //         Ok(cron) => {
                //             if !cron.any() {
                //                 println!("Cron will never match any given time!");
                //                 return;
                //             }

                //             if let Some(start_date) = options.start_date {
                //                 println!("Start date: {start_date}");
                //             } else {
                //                 println!("Start date: None");
                //             }

                //             println!("Upcoming:");
                //             let futures = cron.clone().iter_from(
                //                 if let Some(start_date) = options.start_date {
                //                     if options.catchup || start_date > Utc::now() {
                //                         start_date.into()
                //                     } else {
                //                         Utc::now()
                //                     }
                //                 } else {
                //                     Utc::now()
                //                 },
                //             );
                //             for time in futures.take(10) {
                //                 if !cron.contains(time) {
                //                     println!("Failed check! Cron does not contain {}.", time);
                //                     break;
                //                 }
                //                 if let Some(end_date) = options.end_date {
                //                     if time > end_date {
                //                         break;
                //                     }
                //                 }
                //                 println!("  {}", time.format("%F %R"));
                //             }
                //         }
                //         Err(err) => println!("{err}: {schedule}"),
                //     }
                // } else {
                //     println!("No schedule set");
                // }
            }
            "tasks" => {
                let tasks = get_tasks().read().unwrap();

                println!("{}", serde_json::to_string_pretty(&*tasks).unwrap());
            }
            "edges" => {
                let edges = get_edges().read().unwrap();

                println!("{}", serde_json::to_string_pretty(&*edges).unwrap());
            }
            "graph" => {
                let matches = matches.subcommand_matches("graph").unwrap();
                if let Some(subcommand) = matches.get_one::<String>("graph_type") {
                    let tasks = get_tasks().read().unwrap();
                    let edges = get_edges().read().unwrap();

                    let mut runner = InMemoryRunner::new(&tasks, &edges);
                    runner.enqueue_run("in_memory", "", Utc::now());

                    match subcommand.as_str() {
                        "mermaid" => {
                            let graph = runner.get_mermaid_graph(0);
                            print!("{graph}");
                        }
                        "graphite" => {
                            let graph = runner.get_graphite_graph(0);
                            print!("{}", serde_json::to_string_pretty(&graph).unwrap());
                        }
                        o => {
                            panic!("undefined graph type: {o}");
                        }
                    }
                }
            }
            "hash" => {
                let tasks = get_tasks().read().unwrap();
                let edges = get_edges().read().unwrap();

                let hash = hash_dag(
                    &serde_json::to_string(&*tasks).unwrap(),
                    &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
                );
                print!("{hash}");
            }
            "tree" => {
                let tasks = get_tasks().read().unwrap();
                let edges = get_edges().read().unwrap();

                let mut runner = InMemoryRunner::new(&tasks, &edges);
                let run_id = runner.enqueue_run("in_memory", "", Utc::now());
                let tasks = runner
                    .get_default_tasks()
                    .iter()
                    .filter(|t| runner.get_task_depth(run_id, t.id) == 0)
                    .map(|t| t.id)
                    .collect::<Vec<usize>>();

                let mut output = "DAG\n".to_string();
                let mut task_ids_in_order: Vec<usize> = vec![];

                for (index, child) in tasks.iter().enumerate() {
                    let is_last = index == tasks.len() - 1;

                    let connector = if is_last { "└── " } else { "├── " };
                    task_ids_in_order.push(*child);
                    output.push_str(&runner.get_tree(
                        run_id,
                        *child,
                        1,
                        connector,
                        vec![is_last],
                        &mut task_ids_in_order,
                    ));
                }
                println!("{}", output);
                // println!("{:?}", task_ids_in_order);
            }
            "run" => {
                let matches = matches.subcommand_matches("run").unwrap();
                if let Some(subcommand) = matches.subcommand_name() {
                    match subcommand {
                        "in_memory" => {
                            let tasks = get_tasks().read().unwrap();
                            let edges = get_edges().read().unwrap();

                            let mut runner = InMemoryRunner::new(&tasks.to_vec(), &edges);

                            let run_id = runner.enqueue_run("", "", Utc::now());

                            let default_tasks = runner.get_default_tasks();

                            for task in &default_tasks {
                                let mut visited = HashSet::new();
                                let mut path = vec![];
                                let circular_dependencies = runner.get_circular_dependencies(
                                    run_id,
                                    task.id,
                                    &mut visited,
                                    &mut path,
                                );

                                if let Some(deps) = circular_dependencies {
                                    panic!("{:?}", deps);
                                }
                            }

                            let num_threads = match matches
                                .subcommand_matches("in_memory")
                                .unwrap()
                                .get_one::<String>("num_threads")
                                .unwrap()
                                .as_str()
                            {
                                "max" => max(
                                    usize::from(std::thread::available_parallelism().unwrap()) - 1,
                                    1,
                                ),
                                any => any.parse::<usize>().unwrap(),
                            };

                            let (tx, rx) = channel();

                            let mut thread_count = 0;

                            for _ in 0..num_threads {
                                let mut runner = runner.clone();
                                let tx = tx.clone();

                                if let Some(queued_task) = runner.pop_priority_queue() {
                                    thread::spawn(move || {
                                        let current_executable_path = &env::args().next().unwrap();

                                        runner.work(
                                            run_id,
                                            &queued_task,
                                            current_executable_path.as_str(),
                                        );
                                        tx.send(()).unwrap();
                                    });

                                    thread_count += 1;
                                    if thread_count >= num_threads {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }

                            for _ in rx.iter() {
                                thread_count -= 1;

                                let mut runner = runner.clone();
                                if let Some(queued_task) = runner.pop_priority_queue() {
                                    let tx = tx.clone();

                                    thread::spawn(move || {
                                        let current_executable_path = &env::args().next().unwrap();

                                        runner.work(
                                            run_id,
                                            &queued_task,
                                            current_executable_path.as_str(),
                                        );
                                        tx.send(()).unwrap();
                                    });

                                    thread_count += 1;

                                    if thread_count >= num_threads {
                                        continue;
                                    }
                                }
                                if thread_count == 0 {
                                    drop(tx);
                                    break;
                                }
                            }

                            // let tasks = runner
                            //     .get_default_tasks()
                            //     .iter()
                            //     .filter(|t| runner.get_task_depth(run_id, t.id) == 0)
                            //     .map(|t| t.id)
                            //     .collect::<Vec<usize>>();

                            // let mut output = "DAG\n".to_string();
                            // let mut task_ids_in_order: Vec<usize> = vec![];

                            // for (index, child) in tasks.iter().enumerate() {
                            //     let is_last = index == tasks.len() - 1;

                            //     let connector = if is_last { "└── " } else { "├── " };
                            //     task_ids_in_order.push(*child);
                            //     output.push_str(&runner.get_tree(
                            //         run_id,
                            //         *child,
                            //         1,
                            //         connector,
                            //         vec![is_last],
                            //         &mut task_ids_in_order,
                            //     ));
                            // }
                            // println!("{}", output);
                            // println!("{:?}", task_ids_in_order);
                        }
                        "function" => {
                            let functions = get_functions().read().unwrap();

                            let sub_matches = matches.subcommand_matches("function").unwrap();
                            let function_name =
                                sub_matches.get_one::<String>("function_name").unwrap();
                            let in_arg = sub_matches.get_one::<String>("in_path").unwrap();
                            let out_path_match = sub_matches.get_one::<String>("out_path");

                            if functions.contains_key(function_name) {
                                if let Some(out_path) = out_path_match {
                                    execute_function_using_json_files(
                                        Path::new(in_arg),
                                        Path::new(out_path),
                                        &functions[function_name],
                                    );
                                } else {
                                    execute_function_using_json_str_args(in_arg, &functions[function_name]);
                                }
                            } else {
                                panic!(
                                    "no such function {function_name}\navailable functions: {:#?}",
                                    functions.keys().collect::<Vec<&String>>()
                                )
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn hash_dag(nodes: &str, edges: &[(usize, usize)]) -> String {
    let mut hasher = DefaultHasher::new();
    let mut edges: Vec<&(usize, usize)> = edges.iter().collect();
    edges.sort();

    let to_hash = serde_json::to_string(&nodes).unwrap() + &serde_json::to_string(&edges).unwrap();
    to_hash.hash(&mut hasher);
    to_base62(hasher.finish())
}
