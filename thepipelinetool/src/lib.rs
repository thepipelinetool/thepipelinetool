mod cli;
// mod dag;
// mod options;

pub mod prelude {
    pub use crate::cli::*;
    pub use crate::{
        add_command, add_task, add_task_with_ref, branch, expand, expand_lazy,
    };
    // pub use crate::options::*;
    pub use runner::local::{hash_dag, LocalRunner};
    pub use runner::{DefRunner, Runner};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use task::task::Task;
    pub use task::task_options::TaskOptions;
    pub use task::task_result::TaskResult;
    pub use task::task_status::TaskStatus;
    pub use task::Branch;
    pub use utils::execute_function;
}

use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{BitOr, Shr},
    process::Command,
};

use graph::dag::get_dag;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use task::{task::Task, task_options::TaskOptions, task_ref::TaskRefInner, Branch};
use utils::function_name_as_string;

impl<T, G> Shr<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
    // R: From<(HashSet<usize>, Option<String>)>,
    // Define any constraints necessary for T, G, and R
    // For example, T might need to support a certain operation
    // that is used in the implementation below.
{
    // type Output = R; // Define the output type of the operation
    type Output = TaskRef<G>; // Define the output type of the operation

    fn shr(self, rhs: TaskRef<G>) -> Self::Output {
        // Define the behavior of the shift right operation
        // This could involve manipulating `dag.value` and `rhs.value`
        // and producing a result of type R.
        // let mut dag = get_dag().lock().unwrap();

        // println!("!");

        // println!("{:#?} {:#?}", self.0.task_ids, rhs.0.task_ids);

        seq(&self, &rhs)
    }
}

impl<T, G> BitOr<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
    // R: From<(HashSet<usize>, Option<String>)>,
    // Define any constraints necessary for T, G, and R
    // For example, T might need to support a certain operation
    // that is used in the implementation below.
{
    // type Output = R; // Define the output type of the operation
    type Output = TaskRef<G>; // Define the output type of the operation

    fn bitor(self, rhs: TaskRef<G>) -> Self::Output {
        // Define the behavior of the shift right operation
        // This could involve manipulating `dag.value` and `rhs.value`
        // and producing a result of type R.
        // let mut dag = get_dag().lock().unwrap();

        par(&self, &rhs)
    }
}

impl<T: Serialize> Serialize for TaskRef<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut json_value = json!({
            "upstream_task_id": self.0.task_ids.iter().next().unwrap_or(&0),
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

// impl<T, G, K> Shr<dyn TaskRefTrait<G>> for T
// where
//     T: TaskRef<K>,
//     K: Serialize,
// {
//     type Output = T;

//     fn shr(self, rhs: dyn TaskRefTrait<G>) -> Self::Output {

//         seq([])
//     }
//     // Implement the trait methods here
// }

// impl Dag {
// pub fn set_options( options: DagOptions<'a>) {
//     dag.options = options;
// }

pub fn expand<F, T, G, const N: usize>(
    //
    function: F,
    template_args_vec: &[T; N],
    options: TaskOptions,
) -> [TaskRef<TaskRefInner<G>>; N]
where
    T: Serialize + DeserializeOwned,
    G: Serialize,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    // let mut dag = get_dag().lock().unwrap();
    let mut dag = get_dag().lock().unwrap();

    let function_name = function_name_as_string(&function).to_string();

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: T = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };

    dag.functions
        .insert(function_name.clone(), Box::new(wrapped_function));

    let mut i = 0;

    [(); N].map(|_| {
        let id = dag.nodes.len();

        dag.nodes.insert(
            id,
            Task {
                id,
                function_name: function_name.clone(),
                template_args: serde_json::to_value(&template_args_vec[i]).unwrap(),
                options,
                lazy_expand: false,
                is_dynamic: false,
                is_branch: false,
            },
        );
        i += 1;

        TaskRef(TaskRefInner {
            task_ids: HashSet::from([id]),
            key: None,

            _marker: std::marker::PhantomData,
        })
    })
}

pub fn add_task_with_ref<F, T, G>(
    //
    function: F,
    task_ref: &TaskRef<T>,
    options: TaskOptions,
) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned,
    G: Serialize,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let mut dag = get_dag().lock().unwrap();

    let id = dag.nodes.len();

    let function_name = function_name_as_string(&function).to_string();
    dag.nodes.insert(
        id,
        Task {
            id,
            function_name: function_name.to_string(),
            template_args: serde_json::to_value(task_ref).unwrap(),
            options,
            lazy_expand: false,
            is_dynamic: false,
            is_branch: false,
        },
    );

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: T = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };

    dag.functions
        .insert(function_name, Box::new(wrapped_function));

    TaskRef(TaskRefInner {
        task_ids: HashSet::from([id]),
        key: None,

        _marker: std::marker::PhantomData,
    })
}

pub fn add_task<F, T, G>(function: F, template_args: T, options: TaskOptions) -> TaskRef<G>
where
    T: Serialize + DeserializeOwned,
    G: Serialize,
    F: Fn(T) -> G + 'static + Sync + Send,
{
    let mut dag = get_dag().lock().unwrap();

    let id = dag.nodes.len();

    let function_name = function_name_as_string(&function).to_string();
    dag.nodes.insert(
        id,
        Task {
            id,
            function_name: function_name.to_string(),
            template_args: serde_json::to_value(template_args).unwrap(),
            options,
            lazy_expand: false,
            is_dynamic: false,
            is_branch: false,
        },
    );

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: T = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };

    dag.functions
        .insert(function_name, Box::new(wrapped_function));

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
    options: TaskOptions,
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
    let mut dag = get_dag().lock().unwrap();

    let id = dag.nodes.len();

    let function_name = function_name_as_string(&function).to_string();
    dag.nodes.insert(
        id,
        Task {
            id,
            function_name: function_name.to_string(),
            template_args: serde_json::to_value(template_args).unwrap(),
            options,
            lazy_expand: false,
            is_dynamic: false,
            is_branch: true,
        },
    );

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: K = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: Branch<T> = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };

    dag.functions
        .insert(function_name, Box::new(wrapped_function));

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
    options: TaskOptions,
) -> TaskRef<Vec<G>>
where
    K: Serialize + DeserializeOwned,
    T: Serialize + DeserializeOwned + IntoIterator<Item = K>,
    G: Serialize, // + IntoIterator<Item = T>,
    F: Fn(K) -> G + 'static + Sync + Send,
{
    let mut dag = get_dag().lock().unwrap();

    let id = dag.nodes.len();

    let function_name = function_name_as_string(&function).to_string();
    dag.nodes.insert(
        id,
        Task {
            id,
            function_name: function_name.to_string(),
            template_args: serde_json::to_value(task_ref).unwrap(),
            options,
            lazy_expand: true,
            is_dynamic: false,
            is_branch: false,
        },
    );

    let wrapped_function = move |value: Value| -> Value {
        // Deserialize the Value into T
        let input: K = serde_json::from_value(value).unwrap();
        // Call the original function
        let output: G = function(input);
        // Serialize the G type back into Value
        serde_json::to_value(output).unwrap()
    };

    dag.functions
        .insert(function_name, Box::new(wrapped_function));

    TaskRef(TaskRefInner {
        task_ids: HashSet::from([id]),
        key: None,

        _marker: std::marker::PhantomData,
    })
}

fn seq<T: Serialize, G: Serialize>(a: &TaskRef<T>, b: &TaskRef<G>) -> TaskRef<G> {
    // assert!(!task_refs.is_empty());
    let mut dag = get_dag().lock().unwrap();

    let mut last: usize = 0;
    // for t in 0..a.0.task_refs.len() - 1 {
    for up in a.0.task_ids.iter() {
        for down in b.0.task_ids.iter() {
            dag.edges.insert((*up, *down));
            last = *down;
        }
    }
    // }

    TaskRef(TaskRefInner {
        task_ids: HashSet::from([last]),
        key: None,

        _marker: std::marker::PhantomData,
    })
}

fn par<T: Serialize, G: Serialize>(a: &TaskRef<T>, b: &TaskRef<G>) -> TaskRef<G> {
    let mut task_ids: HashSet<usize> = HashSet::new();

    // for t in task_refs {
    task_ids.extend(&a.0.task_ids);
    task_ids.extend(&b.0.task_ids);
    // }

    TaskRef(TaskRefInner {
        task_ids,
        key: None,
        _marker: std::marker::PhantomData,
    })
}

// pub fn seq<T: Serialize>(task_refs: &[&MyWrapper<T>]) -> MyWrapper<T> {
//     assert!(!task_refs.is_empty());
//     let mut dag = get_dag().lock().unwrap();

//     let mut last: usize = 0;
//     for t in 0..task_refs.len() - 1 {
//         for up in task_refs[t].0.task_ids.iter() {
//             for down in task_refs[t + 1].0.task_ids.iter() {
//                 dag.edges.insert((*up, *down));
//                 last = *down;
//             }
//         }
//     }

//     MyWrapper(TaskRef {
//         task_ids: HashSet::from([last]),
//         key: None,

//         _marker: std::marker::PhantomData,
//     })
// }

// pub fn par<T: Serialize>(task_refs: &[&TaskRef<T>]) -> TaskRef<T> {
//     let mut task_ids: HashSet<usize> = HashSet::new();

//     for t in task_refs {
//         task_ids.extend(&t.task_ids);
//     }

//     TaskRef {
//         task_ids,
//         key: None,
//         _marker: std::marker::PhantomData,
//     }
// }

pub fn add_command(args: Value, options: TaskOptions) -> TaskRef<Value> {
    assert!(args.is_array());

    add_task(run_command, args, options)
}

// pub fn get_initial_mermaid_graph(&self) -> String {
//     let mut runner = LocalRunner::new("", &dag.nodes, &dag.edges);
//     runner.enqueue_run("local", "", Utc::now().into());
//     runner.get_mermaid_graph(&0)
// }

// pub fn hash() -> String {
//     // let dag = get_dag().lock().unwrap();

    
// }
// }

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
