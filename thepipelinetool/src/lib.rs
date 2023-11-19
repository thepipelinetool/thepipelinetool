mod cli;
mod options;

pub mod prelude {
    pub use crate::cli::*;
    pub use crate::options::DagOptions;
    pub use crate::{
        add_command, add_task, add_task_with_ref, branch, expand, expand_lazy, set_catchup,
        set_end_date, set_schedule, set_start_date,
    };
    pub use proc_macro::dag;
    pub use runner::in_memory::InMemoryRunner;
    pub use runner::{blanket::BlanketRunner, Runner};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use task::branch::Branch;
    pub use task::ordered_queued_task::OrderedQueuedTask;
    pub use task::queued_task::QueuedTask;
    pub use task::task_options::TaskOptions;
    pub use task::task_result::TaskResult;
    pub use task::task_status::TaskStatus;
    pub use task::Task;
    pub use utils::execute_function;
}

use chrono::{DateTime, FixedOffset};
use options::DagOptions;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::sync::{OnceLock, RwLock};
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::{BitOr, Shr},
    process::Command,
};
use task::branch::Branch;
use task::Task;
use task::{task_options::TaskOptions, task_ref_inner::TaskRefInner};
use utils::{collector, function_name_as_string};

type StaticTasks = RwLock<Vec<Task>>;
type StaticFunctions = RwLock<HashMap<String, Box<dyn Fn(Value) -> Value + Sync + Send>>>;
type StaticEdges = RwLock<HashSet<(usize, usize)>>;
type StaticOptions = RwLock<DagOptions>;

static TASKS: OnceLock<StaticTasks> = OnceLock::new();
static FUNCTIONS: OnceLock<StaticFunctions> = OnceLock::new();
static EDGES: OnceLock<StaticEdges> = OnceLock::new();
static OPTIONS: OnceLock<StaticOptions> = OnceLock::new();

pub fn get_tasks() -> &'static StaticTasks {
    TASKS.get_or_init(StaticTasks::default)
}

pub fn get_functions() -> &'static StaticFunctions {
    FUNCTIONS.get_or_init(StaticFunctions::default)
}

pub fn get_edges() -> &'static StaticEdges {
    EDGES.get_or_init(StaticEdges::default)
}

pub fn get_options() -> &'static StaticOptions {
    OPTIONS.get_or_init(StaticOptions::default)
}

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

impl<T: Serialize> Serialize for TaskRef<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut json_value = json!({
            "upstream_task_id": self.0.task_ids.iter().next().unwrap(),
        });

        if self.0.key.is_some() {
            json_value["key"] = serde_json::Value::String(self.0.key.clone().unwrap());
        }

        json_value.serialize(serializer)
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

    // let mut tasks = ;

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

pub fn add_command(args: Value, options: &TaskOptions) -> TaskRef<Value> {
    assert!(args.is_array());
    add_task(run_command, args, options)
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

pub fn set_schedule(schedule: &str) {
    get_options().write().unwrap().schedule = Some(schedule.to_string());
}

pub fn set_start_date(start_date: DateTime<FixedOffset>) {
    get_options().write().unwrap().start_date = Some(start_date);
}

pub fn set_end_date(end_date: DateTime<FixedOffset>) {
    get_options().write().unwrap().end_date = Some(end_date);
}

pub fn set_catchup(catchup: bool) {
    get_options().write().unwrap().catchup = catchup;
}
