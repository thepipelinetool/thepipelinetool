use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    env,
    process::Command,
};

use clap::{arg, command, value_parser, Command as CliCommand};
use runner::{
    collector,
    local::{hash_dag, LocalRunner},
    DefRunner, Runner,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use task::{task::Task, task_options::TaskOptions, task_ref::TaskRef, Branch};
use utils::{execute_function, function_name_as_string};
pub struct DAG<'a> {
    pub name: &'a str,
    pub schedule: Option<&'a str>,
    pub nodes: Vec<Task>,
    pub functions: HashMap<String, Box<dyn Fn(Value) -> Value>>,
    pub edges: HashSet<(usize, usize)>,
}

impl<'a> DAG<'a> {
    pub fn new(name: &'a str) -> Self {
        let mut functions: HashMap<String, Box<dyn Fn(Value) -> Value>> = HashMap::new();
        let function_name = function_name_as_string(&collector).to_string();
        functions.insert(function_name.clone(), Box::new(collector));

        Self {
            nodes: Vec::new(),
            functions,
            schedule: None,
            edges: HashSet::new(),
            name,
        }
    }

    pub fn set_schedule(&mut self, schedule: &'a str) {
        self.schedule = Some(schedule);
    }

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
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_command(&mut self, args: Value, options: TaskOptions) -> TaskRef<Value> {
        assert!(args.is_array());

        self.add_task(run_command, args, options)
    }

    pub fn get_initial_mermaid_graph(&self) -> String {
        let mut runner = LocalRunner::new("", &self.nodes, &self.edges);
        runner.enqueue_run("local", "");
        runner.get_mermaid_graph(&0)
    }

    pub fn hash(&self) -> String {
        hash_dag(
            &serde_json::to_string(&self.nodes).unwrap(),
            &self.edges.iter().collect::<Vec<&(usize, usize)>>(),
        )
    }

    pub fn parse_cli(&self) {
        let command = command!()
            .about(format!("CLI Tool for {}", self.name))
            .subcommand(CliCommand::new("tasks").about("Displays tasks"))
            .subcommand(CliCommand::new("edges").about("Displays edges"))
            .subcommand(CliCommand::new("graph").about("Displays graph"))
            .subcommand(CliCommand::new("hash").about("Displays hash"))
            .subcommand(CliCommand::new("tree").about("Displays tree"))
            .subcommand(
                CliCommand::new("run")
                    .arg_required_else_help(true)
                    .subcommand(
                        CliCommand::new("local").about("Runs dag locally").arg(
                            arg!(
                                [mode] "Mode for running locally"
                            )
                            .required(false)
                            .value_parser(value_parser!(String))
                            .default_values(["max", "--blocking"])
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
                                    <out_path> "Output file"
                                )
                                .required(true),
                            )
                            .arg(
                                arg!(
                                    <in_path> "Input file"
                                )
                                .required(true),
                            ),
                    ),
            );

        let matches = command.get_matches();

        if let Some(subcommand) = matches.subcommand_name() {
            match subcommand {
                "tasks" => {
                    println!("{}", serde_json::to_string_pretty(&self.nodes).unwrap());
                }
                "edges" => {
                    println!("{}", serde_json::to_string_pretty(&self.edges).unwrap());
                }
                "graph" => {
                    print!("{}", self.get_initial_mermaid_graph());
                }
                "hash" => {
                    print!("{}", self.hash());
                }
                "tree" => {
                    let mut runner = LocalRunner::new("", &self.nodes, &self.edges);
                    let dag_run_id = runner.enqueue_run("local", "");
                    let tasks = runner
                        .get_default_tasks()
                        .iter()
                        .filter(|t| runner.get_upstream(&dag_run_id, &t.id).is_empty())
                        .map(|t| t.id)
                        .collect::<Vec<usize>>();

                    let mut output = format!("{}\n", self.name);
                    let mut ts: Vec<usize> = vec![];

                    for (index, child) in tasks.iter().enumerate() {
                        let is_last = index == tasks.len() - 1;

                        let connector = if is_last { "└── " } else { "├── " };
                        ts.push(*child);
                        output.push_str(&runner.get_tree(
                            &dag_run_id,
                            child,
                            1,
                            connector,
                            vec![is_last],
                            &mut ts,
                        ));
                    }
                    println!("{}", output);
                    println!("{:?}", ts);
                }
                "run" => {
                    let matches = matches.subcommand_matches("run").unwrap();
                    if let Some(subcommand) = matches.subcommand_name() {
                        match subcommand {
                            "local" => {
                                let sub_matches = matches.subcommand_matches("local").unwrap();
                                let mode = sub_matches.get_one::<String>("mode").unwrap();

                                let max_threads = max(
                                    usize::from(std::thread::available_parallelism().unwrap()) - 1,
                                    1,
                                );
                                let thread_count = match mode.as_str() {
                                    "--blocking" => 1,
                                    "max" => max_threads,
                                    _ => mode.parse::<usize>().unwrap(),
                                };
                                LocalRunner::new("", &self.nodes, &self.edges)
                                    .run_dag_local(thread_count);
                            }
                            "function" => {
                                let sub_matches = matches.subcommand_matches("function").unwrap();
                                let function_name =
                                    sub_matches.get_one::<String>("function_name").unwrap();
                                let in_path = sub_matches.get_one::<String>("in_path").unwrap();
                                let out_path = sub_matches.get_one::<String>("out_path").unwrap();

                                if self.functions.contains_key(function_name) {
                                    execute_function(
                                        in_path,
                                        out_path,
                                        &self.functions[function_name],
                                    );
                                } else {
                                    panic!(
                                        "no such function {function_name}\navailable functions: {:#?}",
                                        self.functions.keys().collect::<Vec<&String>>()
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
