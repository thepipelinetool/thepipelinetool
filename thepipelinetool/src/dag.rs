use std::{
    collections::{HashMap, HashSet},
    process::Command,
};

use chrono::Utc;
use runner::{
    collector,
    local::{hash_dag, LocalRunner},
    DefRunner,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use task::{task::Task, task_options::TaskOptions, task_ref::TaskRef, Branch};
use utils::function_name_as_string;

use crate::options::DagOptions;
pub struct DAG {
    pub nodes: Vec<Task>,
    pub functions: HashMap<String, Box<dyn Fn(Value) -> Value>>,
    pub edges: HashSet<(usize, usize)>,
    pub options: DagOptions,
}

impl DAG {
    pub fn new() -> Self {
        let mut functions: HashMap<String, Box<dyn Fn(Value) -> Value>> = HashMap::new();
        let function_name = function_name_as_string(&collector).to_string();
        functions.insert(function_name.clone(), Box::new(collector));

        Self {
            nodes: Vec::new(),
            functions,
            edges: HashSet::new(),
            options: DagOptions::default(),
        }
    }

    // pub fn set_options(&mut self, options: DagOptions<'a>) {
    //     self.options = options;
    // }

    pub fn expand<F, T, G, const N: usize>(
        &mut self,
        function: F,
        template_args_vec: &[T; N],
        options: TaskOptions,
    ) -> [TaskRef<G>; N]
    where
        T: Serialize + DeserializeOwned,
        G: Serialize,
        F: Fn(T) -> G + 'static,
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

        self.functions
            .insert(function_name.clone(), Box::new(wrapped_function));

        let mut i = 0;

        [(); N].map(|_| {
            let id = self.nodes.len();

            self.nodes.insert(
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

            TaskRef {
                task_ids: HashSet::from([id]),
                key: None,

                _marker: std::marker::PhantomData,
            }
        })
    }

    pub fn add_task_with_ref<F, T, G>(
        &mut self,
        function: F,
        task_ref: &TaskRef<T>,
        options: TaskOptions,
    ) -> TaskRef<G>
    where
        T: Serialize + DeserializeOwned,
        G: Serialize,
        F: Fn(T) -> G + 'static,
    {
        let id = self.nodes.len();

        let function_name = function_name_as_string(&function).to_string();
        self.nodes.insert(
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

        self.functions
            .insert(function_name, Box::new(wrapped_function));

        TaskRef {
            task_ids: HashSet::from([id]),
            key: None,

            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_task<F, T, G>(
        &mut self,
        function: F,
        template_args: T,
        options: TaskOptions,
    ) -> TaskRef<G>
    where
        T: Serialize + DeserializeOwned,
        G: Serialize,
        F: Fn(T) -> G + 'static,
    {
        let id = self.nodes.len();

        let function_name = function_name_as_string(&function).to_string();
        self.nodes.insert(
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

        self.functions
            .insert(function_name, Box::new(wrapped_function));

        TaskRef {
            task_ids: HashSet::from([id]),
            key: None,

            _marker: std::marker::PhantomData,
        }
    }

    pub fn branch<F, K, T, L, J, R, M>(
        &mut self,
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
        F: Fn(K) -> Branch<T> + 'static,
        L: Fn(T) -> J + 'static,
        R: Fn(T) -> M + 'static,
    {
        let id = self.nodes.len();

        let function_name = function_name_as_string(&function).to_string();
        self.nodes.insert(
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

        self.functions
            .insert(function_name, Box::new(wrapped_function));

        let b = TaskRef::<T> {
            task_ids: HashSet::from([id]),
            key: None,

            _marker: std::marker::PhantomData,
        };

        (
            self.add_task_with_ref(left, &b, options),
            self.add_task_with_ref(right, &b, options),
        )
    }

    pub fn expand_lazy<K, F, T, G>(
        &mut self,
        function: F,
        task_ref: &TaskRef<T>,
        options: TaskOptions,
    ) -> TaskRef<Vec<G>>
    where
        K: Serialize + DeserializeOwned,
        T: Serialize + DeserializeOwned + IntoIterator<Item = K>,
        G: Serialize, // + IntoIterator<Item = T>,
        F: Fn(K) -> G + 'static,
    {
        let id = self.nodes.len();

        let function_name = function_name_as_string(&function).to_string();
        self.nodes.insert(
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

        self.functions
            .insert(function_name, Box::new(wrapped_function));

        TaskRef {
            task_ids: HashSet::from([id]),
            key: None,

            _marker: std::marker::PhantomData,
        }
    }

    pub fn seq<T: Serialize>(&mut self, task_refs: &[&TaskRef<T>]) -> TaskRef<T> {
        assert!(!task_refs.is_empty());

        let mut last: usize = 0;
        for t in 0..task_refs.len() - 1 {
            for up in task_refs[t].task_ids.iter() {
                for down in task_refs[t + 1].task_ids.iter() {
                    self.edges.insert((*up, *down));
                    last = *down;
                }
            }
        }

        TaskRef {
            task_ids: HashSet::from([last]),
            key: None,

            _marker: std::marker::PhantomData,
        }
    }

    pub fn par<T: Serialize>(&mut self, task_refs: &[&TaskRef<T>]) -> TaskRef<T> {
        let mut task_ids: HashSet<usize> = HashSet::new();

        for t in task_refs {
            task_ids.extend(&t.task_ids);
        }

        TaskRef {
            task_ids,
            key: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_command(&mut self, args: Value, options: TaskOptions) -> TaskRef<Value> {
        assert!(args.is_array());

        self.add_task(run_command, args, options)
    }

    // pub fn get_initial_mermaid_graph(&self) -> String {
    //     let mut runner = LocalRunner::new("", &self.nodes, &self.edges);
    //     runner.enqueue_run("local", "", Utc::now().into());
    //     runner.get_mermaid_graph(&0)
    // }

    pub fn get_graphite_mermaid_graph(&self) -> Vec<Value> {
        let mut runner = LocalRunner::new("", &self.nodes, &self.edges);
        runner.enqueue_run("local", "", Utc::now());
        runner.get_graphite_graph(&0)
    }

    pub fn hash(&self) -> String {
        hash_dag(
            &serde_json::to_string(&self.nodes).unwrap(),
            &self.edges.iter().collect::<Vec<&(usize, usize)>>(),
        )
    }
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
