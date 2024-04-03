use std::{collections::HashMap, fs::File, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thepipelinetool_core::{
    _lazy_task_ref,
    dev::{
        bash_operator, get_edges, get_tasks, Operator, TaskOptions, _add_task_with_function_name,
        _expand_lazy_with_function_name, _register_function_with_name, get_functions,
        params::params_operator, print::print_operator, register_function, UPSTREAM_TASK_ID_KEY,
        UPSTREAM_TASK_RESULT_KEY,
    },
};
use thepipelinetool_utils::{collector, function_name_as_string};

// use crate::create_template_args;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct YamlTaskTemplate {
    #[serde(default)]
    pub name: String,

    // #[serde(default)]
    // pub function_name: String,
    #[serde(default)]
    pub args: Value,

    #[serde(default)]
    pub options: TaskOptions,

    #[serde(default)]
    pub lazy_expand: bool,

    #[serde(default)]
    pub is_branch: bool,

    #[serde(default = "default_operator")]
    pub operator: String,

    #[serde(default)]
    pub depends_on: Vec<String>,
}

fn default_operator() -> String {
    function_name_as_string(bash_operator).to_string()
}

// impl Default for YamlTaskTemplate {
//     fn default() -> Self {
//         Self {
//             name: "".to_string(),
//             // function_name: "".to_string(),
//             args: json!([]),
//             options: Default::default(),
//             lazy_expand: false,
//             is_branch: false,
//             operator: ,
//             depends_on: Vec::new(),
//         }
//     }
// }

pub fn read_from_yaml(dag_path: &Path) {
    let value: Value = serde_yaml::from_reader(File::open(dag_path).unwrap()).unwrap();

    if value.as_object().unwrap().contains_key("tasks") {
        let tasks = value["tasks"].as_object().unwrap();
        let mut task_id_by_name: HashMap<String, usize> = HashMap::new();
        let base_id = get_tasks().read().unwrap().len();

        let mut template_tasks: Vec<YamlTaskTemplate> = tasks
            .iter()
            .rev()
            .enumerate()
            .map(|(i, (k, v))| {
                task_id_by_name.insert(k.to_string(), base_id + i);

                let mut template: YamlTaskTemplate = serde_json::from_value(v.clone()).unwrap();
                template.name = k.to_string();
                template
            })
            .collect();

        for template_task in template_tasks.iter_mut() {
            let id = *task_id_by_name.get(&template_task.name).unwrap();

            // create edges
            let depends_on: Vec<usize> = template_task
                .depends_on
                .iter()
                .map(|dependency| {
                    let upstream_id = *task_id_by_name
                        .get(dependency)
                        .unwrap_or_else(|| panic!("upstream task '{dependency}' missing"));
                    get_edges().write().unwrap().insert((upstream_id, id));
                    upstream_id
                })
                .collect();

            // load operators
            let operator = &serde_json::from_value::<Operator>(json!(template_task.operator)).ok();

            if let Some(operator) = operator {
                _register_function_with_name(
                    match operator {
                        Operator::BashOperator => bash_operator,
                        Operator::ParamsOperator => params_operator,
                        Operator::PrintOperator => print_operator,
                    },
                    &template_task.operator,
                );
            }

            let contains_key = get_functions()
                .read()
                .unwrap()
                .contains_key(&template_task.operator);

            // load tasks
            if contains_key {
                if template_task.lazy_expand {
                    assert!(depends_on.len() == 1);

                    register_function(collector);
                    _expand_lazy_with_function_name::<Value, Vec<Value>, Value>(
                        &_lazy_task_ref(depends_on[0]),
                        &template_task.options,
                        &template_task.name,
                        &template_task.operator,
                    );
                } else {
                    let template_args = match operator {
                        Some(Operator::BashOperator) => {
                            create_bash_args(id, &template_task.args, &task_id_by_name)
                        }
                        _ => template_task.args.clone(),
                    };

                    _add_task_with_function_name::<Value, Value>(
                        template_args,
                        &template_task.options,
                        &template_task.name,
                        &template_task.operator,
                    );
                }
            } else {
                let function_name = &template_task.operator;
                panic!(
                    "no such function '{function_name}'\navailable functions: {:#?}",
                    get_functions()
                        .read()
                        .unwrap()
                        .keys()
                        .collect::<Vec<&String>>()
                )
            }
        }
    }
}

fn _get_task_id_by_name(name: &str) -> usize {
    get_tasks()
        .read()
        .unwrap()
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("missing task {name}"))
        .id
}

// TODO check for multiple matches, rework
fn create_bash_args(
    task_id: usize,
    args: &Value,
    task_id_by_name: &HashMap<String, usize>,
) -> Value {
    let mut temp_args = json!({});

    let mut args = args;

    if args.is_array() {
        args = &args.as_array().unwrap()[0];
    }

    assert!(args.is_string());

    let mut arg = args.as_str().unwrap().trim().to_string();

    loop {
        let (left, right) = (arg.find("{{"), arg.find("}}"));

        if left.is_none() || right.is_none() {
            break;
        }
        let (left, right) = (left.unwrap(), right.unwrap());

        let chunks: Vec<&str> = arg[(left + 2)..(right)].trim().split('.').collect();

        let task_name = chunks[0];
        let upstream_id = if let Some(id) = task_id_by_name.get(task_name) {
            *id
        } else {
            _get_task_id_by_name(task_name)
        };

        let to_replace = &arg[left..(right + 2)].to_string();

        temp_args[to_replace] = json!({
            UPSTREAM_TASK_ID_KEY: upstream_id
        });

        if chunks.len() > 1 {
            temp_args[to_replace][UPSTREAM_TASK_RESULT_KEY] = chunks[1].into();
        }
        get_edges().write().unwrap().insert((upstream_id, task_id));
        arg.replace_range(left..(right + 2), "");
    }

    temp_args["_original_command"] = args.as_str().unwrap().trim().to_string().into();

    temp_args
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;
    use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

    use crate::yaml::create_bash_args;

    #[test]
    fn test_create_bash_args() {
        let mut task_id_by_name: HashMap<String, usize> = HashMap::new();

        task_id_by_name.insert("t1".into(), 0);
        task_id_by_name.insert("t2".into(), 1);
        task_id_by_name.insert("t3".into(), 2);

        assert_eq!(
            json!({
                "{{  t1 }}": { UPSTREAM_TASK_ID_KEY: 0 }
            }),
            create_bash_args(1, &json!("echo {{  t1 }}"), &task_id_by_name)
        );
        assert_eq!(
            json!({
                "{{  t1 }}": { UPSTREAM_TASK_ID_KEY: 0 },
                "{{t2}}": { UPSTREAM_TASK_ID_KEY: 1 }
            }),
            create_bash_args(1, &json!("echo {{  t1 }}{{t2}}"), &task_id_by_name)
        );
        assert_eq!(
            json!({
                "{{  t1 }}": { UPSTREAM_TASK_ID_KEY: 0 },
                "{{t2}}": { UPSTREAM_TASK_ID_KEY: 1 },
                "{{t3.data}}": { UPSTREAM_TASK_ID_KEY: 2, UPSTREAM_TASK_RESULT_KEY: "data" }
            }),
            create_bash_args(
                1,
                &json!("echo {{  t1 }}{{t2}}{{t3.data}}"),
                &task_id_by_name
            )
        );
    }
}
//         assert_eq!(
//             json!({
//                 UPSTREAM_TASK_ID_KEY: 0,
//                 UPSTREAM_TASK_RESULT_KEY: "test"
//             }),
//             create_template_args(1, &json!("{{t1.test}}"), &task_id_by_name)
//         );

//         assert_eq!(
//             json!(["echo", {
//                 UPSTREAM_TASK_ID_KEY: 0
//             }]),
//             create_template_args(1, &json!(["echo", "{{ t1  }}"]), &task_id_by_name,)
//         );
//         assert_eq!(
//             json!({"data": {
//                 UPSTREAM_TASK_ID_KEY: 0,
//                 UPSTREAM_TASK_RESULT_KEY: "test"
//             }}),
//             create_template_args(1, &json!({"data": "{{ t1.test   }}"}), &task_id_by_name,)
//         );
//     }
// }

#[cfg(test)]
mod test {
    use std::path::Path;

    #[test]
    fn test() {
        assert!(Path::new("simple.yaml").with_extension("") == Path::new("simple"));
    }
}
