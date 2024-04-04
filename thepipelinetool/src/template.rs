use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thepipelinetool_core::dev::{bash_operator, get_edges, get_tasks, TaskOptions};
use thepipelinetool_utils::{
    function_name_as_string, UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY,
};

fn default_operator() -> String {
    function_name_as_string(bash_operator).to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateTask {
    #[serde(default)]
    pub name: String,

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

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct TemplateTaskArgs {
//     #[serde(default)]
//     pub args: Value,
// }

fn _get_task_id_by_name(name: &str) -> usize {
    get_tasks()
        .read()
        .unwrap()
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("missing task {name}"))
        .id
}

pub fn create_template_args_from_string(
    task_id: usize,
    string: &str,
    task_id_by_name: &HashMap<String, usize>,
) -> Value {
    assert!(!string.trim().is_empty());

    let mut temp_args = json!({});
    let string = &mut string.to_string();

    loop {
        let (left, right) = (string.find("{{"), string.find("}}"));

        if left.is_none() || right.is_none() {
            break;
        }
        let (left, right) = (left.unwrap(), right.unwrap());

        let chunks: Vec<&str> = string[(left + 2)..(right)].trim().split('.').collect();

        let task_name = chunks[0];
        let upstream_id = if let Some(id) = task_id_by_name.get(task_name) {
            *id
        } else {
            _get_task_id_by_name(task_name)
        };

        let to_replace = &string[left..(right + 2)].to_string();

        temp_args[to_replace] = json!({
            UPSTREAM_TASK_ID_KEY: upstream_id
        });

        if chunks.len() > 1 {
            temp_args[to_replace][UPSTREAM_TASK_RESULT_KEY] = chunks[1].into();
        }
        get_edges().write().unwrap().insert((upstream_id, task_id));
        string.replace_range(left..(right + 2), "");
    }

    temp_args["_original_command"] = string.to_string().into();

    temp_args
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;
    use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

    use crate::template::create_template_args_from_string;

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
            create_template_args_from_string(1, "echo {{  t1 }}", &task_id_by_name)
        );
        assert_eq!(
            json!({
                "{{  t1 }}": { UPSTREAM_TASK_ID_KEY: 0 },
                "{{t2}}": { UPSTREAM_TASK_ID_KEY: 1 }
            }),
            create_template_args_from_string(1, "echo {{  t1 }}{{t2}}", &task_id_by_name)
        );
        assert_eq!(
            json!({
                "{{  t1 }}": { UPSTREAM_TASK_ID_KEY: 0 },
                "{{t2}}": { UPSTREAM_TASK_ID_KEY: 1 },
                "{{t3.data}}": { UPSTREAM_TASK_ID_KEY: 2, UPSTREAM_TASK_RESULT_KEY: "data" }
            }),
            create_template_args_from_string(
                1,
                "echo {{  t1 }}{{t2}}{{t3.data}}",
                &task_id_by_name
            )
        );
    }
}
