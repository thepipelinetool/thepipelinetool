use std::{collections::HashMap, fs::File, path::Path};

use serde_json::{json, Value};
use thepipelinetool_core::{
    _lazy_task_ref,
    dev::{
        bash_operator, get_edges, get_tasks, Operator, _add_task_with_function_name,
        _expand_lazy_with_function_name, _register_function_with_name, bash::TemplateBashTaskArgs,
        get_functions, params::params_operator, print::print_operator, register_function,
    },
};
use thepipelinetool_utils::collector;

use crate::template::{create_template_args_from_string, TemplateTask};

// use crate::create_template_args;

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

        let mut template_tasks: Vec<(TemplateTask, Value)> = tasks
            .iter()
            .rev()
            .enumerate()
            .map(|(i, (k, v))| {
                task_id_by_name.insert(k.to_string(), base_id + i);

                let mut template: TemplateTask = serde_json::from_value(v.clone()).unwrap();
                template.name = k.to_string();
                (template, v.clone())
            })
            .collect();

        for (template_task, value) in template_tasks.iter_mut() {
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
                    let default_args = json!({});

                    // generate template args dependant on operator type
                    let template_args = match operator {
                        Some(Operator::BashOperator) => create_template_args_from_string(
                            id,
                            &serde_json::from_value::<TemplateBashTaskArgs>(value.clone())
                                .unwrap()
                                .script,
                            &task_id_by_name,
                        ),
                        _ => value
                            .as_object()
                            .unwrap()
                            .get("args")
                            .unwrap_or(&default_args)
                            .clone(),
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
