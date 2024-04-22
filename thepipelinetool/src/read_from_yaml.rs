use std::collections::HashMap;

use serde_json::{json, Value};
use thepipelinetool_core::dev::{
    get_edges, get_tasks, Operator, _add_task_with_function_name, _expand_lazy_with_function_name,
    _lazy_task_ref,
};

use crate::templating::{create_template_args_by_operator, TemplateTask};

pub fn read_from_yaml(value: Value) {
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
            let use_trigger_params =
                value.is_object() && value["use_trigger_params"].as_bool().unwrap_or(false);

            // create edges
            let depends_on: Vec<usize> = template_task
                .depends_on
                .iter()
                .map(|dependency| {
                    // TODO be able to depend on tasks defined in rust pipeline?
                    let upstream_id = *task_id_by_name
                        .get(dependency)
                        .unwrap_or_else(|| panic!("upstream task '{dependency}' missing"));
                    get_edges().write().unwrap().insert((upstream_id, id));
                    upstream_id
                })
                .collect();

            // // try parse operator
            let operator = &serde_json::from_value::<Operator>(json!(template_task.operator)).ok();
            // // register built-in operators if used
            // if let Some(built_in_operator) = operator {
            //     _register_function_with_name(
            //         match built_in_operator {
            //             Operator::BashOperator => bash_operator,
            //             Operator::ParamsOperator => params_operator,
            //             Operator::PrintOperator => print_operator,
            //             Operator::AssertOperator => assert_operator,
            //             Operator::PythonOperator => python_operator,
            //         },
            //         &template_task.operator,
            //     );
            // }

            // if !function_with_name_exists(&template_task.operator) {
            //     panic!(
            //         "no such function '{}'\navailable functions: {:#?}",
            //         &template_task.operator,
            //         get_functions()
            //             .read()
            //             .unwrap()
            //             .keys()
            //             .collect::<Vec<&String>>()
            //     );
            // }

            if template_task.lazy_expand {
                assert!(depends_on.len() == 1);
                _expand_lazy_with_function_name::<Value, Vec<Value>, Value>(
                    &_lazy_task_ref(depends_on[0]),
                    &template_task.options,
                    &template_task.name,
                    &template_task.operator,
                );
            } else {
                _add_task_with_function_name::<Value, Value>(
                    create_template_args_by_operator(id, value, operator, &task_id_by_name),
                    &template_task.options,
                    &template_task.name,
                    &template_task.operator,
                    use_trigger_params,
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    #[test]
    fn test() {
        assert!(Path::new("simple.yaml").with_extension("") == Path::new("simple"));
    }
}
