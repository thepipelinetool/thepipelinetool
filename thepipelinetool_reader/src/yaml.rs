use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thepipelinetool_task::task_options::TaskOptions;
use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskTemplate {
    #[serde(default)]
    pub name: String,
    
    #[serde(default)]
    pub function_name: String,

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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Operator {
    Bash,
}

impl Default for Operator {
    fn default() -> Self {
        Operator::Bash
    }
}

impl Default for TaskTemplate {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            function_name: "".to_string(),
            args: Value::Null,
            options: Default::default(),
            lazy_expand: false,
            is_branch: false,
            operator: Operator::default(),
        }
    }
}

pub fn read_from_yaml(dag_path: &Path) -> (Vec<TaskTemplate>, HashSet<(usize, usize)>) {
    let value: Value = serde_yaml::from_reader(File::open(dag_path).unwrap()).unwrap();

    if !value.as_object().unwrap().contains_key("tasks") {
        panic!("invalid yaml (missing tasks)");
    }
    let tasks = value["tasks"].as_object().unwrap();
    let mut task_id_by_name: HashMap<String, usize> = HashMap::new();

    let mut template_tasks: Vec<TaskTemplate> = tasks
        .iter()
        .enumerate()
        .map(|(i, (k, v))| {
            task_id_by_name.insert(k.to_string(), i);

            let mut template: TaskTemplate = serde_json::from_value(v.clone()).unwrap();
            template.name = k.to_string();
            template
        })
        .collect();

    let mut edges: HashSet<(usize, usize)> = HashSet::new();

    for (id, template_task) in template_tasks.iter_mut().enumerate() {
        // TODO check for non-array args
        template_task.args = template_task
            .args
            .as_array()
            .unwrap()
            .iter()
            .map(|f| create_template_args(id, &f.as_str().unwrap(), &task_id_by_name, &mut edges))
            .collect();
    }

    (template_tasks, edges)
}

fn create_template_args(
    task_id: usize,
    arg: &str,
    task_id_by_name: &HashMap<String, usize>,
    edges: &mut HashSet<(usize, usize)>,
) -> Value {
    let left = arg.find("{{");
    let right = arg.find("}}");

    match (left, right) {
        (Some(left), Some(right)) => {
            let chunks: Vec<&str> = arg[(left + 2)..right].trim().split(".").collect();
            let mut template_args = json!({});
            let task_name = chunks[0];

            let upstream_id = *task_id_by_name
                .get(task_name)
                .expect(&format!("missing task {task_name}"));

            template_args[UPSTREAM_TASK_ID_KEY] = upstream_id.into();

            if chunks.len() > 1 {
                template_args[UPSTREAM_TASK_RESULT_KEY] = chunks[1].into();
            }

            edges.insert((upstream_id, task_id));

            return template_args;
        }
        _ => {
            return arg.into();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use serde_json::json;
    use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

    use crate::yaml::create_template_args;

    #[test]
    fn test_resolve_template_args() {
        let mut task_id_by_name: HashMap<String, usize> = HashMap::new();
        let mut edges: HashSet<(usize, usize)> = HashSet::new();

        task_id_by_name.insert("t1".into(), 0);

        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0
            }),
            create_template_args(1, "{{t1}}", &task_id_by_name, &mut edges)
        );
        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0,
                UPSTREAM_TASK_RESULT_KEY: "test"
            }),
            create_template_args(1, "{{t1.test}}", &task_id_by_name, &mut edges)
        );
        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0
            }),
            create_template_args(1, "{{  t1   }}", &task_id_by_name, &mut edges)
        );
        assert_eq!(
            json!({
                UPSTREAM_TASK_ID_KEY: 0,
                UPSTREAM_TASK_RESULT_KEY: "test"
            }),
            create_template_args(1, "{{   t1.test   }}", &task_id_by_name, &mut edges)
        );
    }
}
