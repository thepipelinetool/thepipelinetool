use std::{collections::HashMap, fs::File, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thepipelinetool_task::task_options::TaskOptions;
use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TaskTemplate {
    #[serde(default)]
    pub function_name: String,

    #[serde(default)]
    pub command: Vec<String>,

    #[serde(default)]
    pub options: TaskOptions,

    #[serde(default)]
    pub lazy_expand: bool,

    #[serde(default)]
    pub is_branch: bool,
}

impl Default for TaskTemplate {
    fn default() -> Self {
        Self {
            function_name: "".to_string(),
            command: Vec::new(),
            options: Default::default(),
            lazy_expand: false,
            is_branch: false,
        }
    }
}

pub fn read_from_yaml(dag_path: &Path) -> Vec<Value> {
    let value: Value = serde_yaml::from_reader(File::open(dag_path).unwrap()).unwrap();

    if !value.as_object().unwrap().contains_key("tasks") {
        panic!("invalid yaml (missing tasks)");
    }
    let tasks = value["tasks"].as_object().unwrap();
    let mut task_id_by_name: HashMap<String, usize> = HashMap::new();

    let task_templates: Vec<TaskTemplate> = tasks
        .iter()
        .enumerate()
        .map(|(i, (k, v))| {
            task_id_by_name.insert(k.to_string(), i);

            serde_json::from_value(v.clone()).unwrap()
        })
        .collect();

    task_templates
        .iter()
        .map(|task_template| {
            let command: Vec<Value> = task_template
                .command
                .iter()
                .map(|f| create_template_args(f, &task_id_by_name))
                .collect();
            let template_args = serde_json::to_value(command).unwrap();
            template_args
            // add_task(run_command, template_args, &TaskOptions::default());
        })
        .collect()
}

fn create_template_args(arg: &str, task_id_by_name: &HashMap<String, usize>) -> Value {
    let left = arg.find("{{");
    let right = arg.find("}}");

    match (left, right) {
        (Some(left), Some(right)) => {
            let chunks: Vec<&str> = arg[(left + 2)..right].trim().split(".").collect();
            let mut template_args = json!({});
            let task_name = chunks[0];

            template_args[UPSTREAM_TASK_ID_KEY] = (*task_id_by_name
                .get(task_name)
                .expect(&format!("missing task {task_name}")))
            .into();

            if chunks.len() > 1 {
                template_args[UPSTREAM_TASK_RESULT_KEY] = chunks[1].into();
            }

            return template_args;
        }
        _ => {
            return arg.into();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;
    use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

    use crate::yaml::create_template_args;

    #[test]
    fn test_resolve_template_args() {
        let mut task_id_by_name: HashMap<String, usize> = HashMap::new();

        task_id_by_name.insert("t1".into(), 0);

        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0
            }),
            create_template_args("{{t1}}", &task_id_by_name)
        );
        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0,
                UPSTREAM_TASK_RESULT_KEY: "test"
            }),
            create_template_args("{{t1.test}}", &task_id_by_name)
        );
        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0
            }),
            create_template_args("{{  t1   }}", &task_id_by_name)
        );
        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0,
                UPSTREAM_TASK_RESULT_KEY: "test"
            }),
            create_template_args("{{   t1.test   }}", &task_id_by_name)
        );
    }
}
