use std::{collections::HashMap, fs::File, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thepipelinetool::dev::{
    add_named_task, bash_operator, get_edges, get_tasks, Operator, TaskOptions,
    UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskTemplate {
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

    #[serde(default)]
    pub operator: Operator,

    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl Default for TaskTemplate {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            // function_name: "".to_string(),
            args: json!([]),
            options: Default::default(),
            lazy_expand: false,
            is_branch: false,
            operator: Operator::default(),
            depends_on: Vec::new(),
        }
    }
}

pub fn read_from_yaml(dag_path: &Path) {
    let value: Value = serde_yaml::from_reader(File::open(dag_path).unwrap()).unwrap();

    if value.as_object().unwrap().contains_key("tasks") {
        let tasks = value["tasks"].as_object().unwrap();
        let mut task_id_by_name: HashMap<String, usize> = HashMap::new();
        let base_id = get_tasks().read().unwrap().len();

        let mut template_tasks: Vec<TaskTemplate> = tasks
            .iter()
            .enumerate()
            .map(|(i, (k, v))| {
                task_id_by_name.insert(k.to_string(), base_id + i);

                let mut template: TaskTemplate = serde_json::from_value(v.clone()).unwrap();
                template.name = k.to_string();
                template
            })
            .collect();

        for template_task in template_tasks.iter_mut() {
            let id = *task_id_by_name.get(&template_task.name).unwrap();
            let template_args = create_template_args(id, &template_task.args, &task_id_by_name);

            for dependency in template_task.depends_on.iter() {
                get_edges().write().unwrap().insert((
                    *task_id_by_name
                        .get(dependency)
                        .unwrap_or_else(|| panic!("upstream task '{dependency}' missing")),
                    id,
                ));
            }

            match template_task.operator {
                Operator::Bash => {
                    add_named_task(
                        bash_operator,
                        template_args,
                        &template_task.options,
                        &template_task.name,
                    );
                } // Operator::Papermill => {
                  //     add_named_task(
                  //         papermill_operator,
                  //         serde_json::from_value(template_task.args).unwrap(),
                  //         &template_task.options,
                  //         &template_task.name,
                  //     );
                  // }
            }
        }
    }
}

// TODO check for multiple matches, rework
fn create_template_args(
    task_id: usize,
    args: &Value,
    task_id_by_name: &HashMap<String, usize>,
) -> Value {
    let args = &mut args.as_array().unwrap();
    let mut temp_args = vec![];

    for i in 0..args.len() {
        if args[i].is_string() {
            let arg = args[i].as_str().unwrap().trim();

            if arg.starts_with("{{") && arg.ends_with("}}") {
                let chunks: Vec<&str> = arg[2..(arg.len() - 2)].trim().split('.').collect();

                let mut template_args = json!({});
                let task_name = chunks[0];
                let upstream_id = if let Some(id) = task_id_by_name.get(task_name) {
                    *id
                } else {
                    get_tasks()
                        .read()
                        .unwrap()
                        .iter()
                        .find(|t| t.name == task_name)
                        .unwrap_or_else(|| panic!("missing task {task_name}"))
                        .id
                };

                template_args[UPSTREAM_TASK_ID_KEY] = upstream_id.into();

                if chunks.len() > 1 {
                    template_args[UPSTREAM_TASK_RESULT_KEY] = chunks[1].into();
                }
                get_edges().write().unwrap().insert((upstream_id, task_id));

                temp_args.push(template_args);
            } else {
                temp_args.push(args[i].clone());
            }
        } else {
            temp_args.push(args[i].clone());
        }
    }

    json!(temp_args)
}

// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;

//     use serde_json::json;
//     use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

//     use crate::yaml::create_template_args;

//     #[test]
//     fn test_resolve_template_args() {
//         let mut task_id_by_name: HashMap<String, usize> = HashMap::new();

//         task_id_by_name.insert("t1".into(), 0);

//         assert_eq!(
//             json!({
//                 UPSTREAM_TASK_ID_KEY: 0
//             }),
//             create_template_args(1, &json!("{{  t1 }}"), &task_id_by_name)
//         );
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
