use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thepipelinetool_core::dev::{
    bash::TemplateBashTaskArgs, bash_operator, get_edges, get_id_by_task_name, Operator,
    TaskOptions, ORIGINAL_STRING_KEY,
};
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

const LEFT_INTERPOLATION_IDENTIFIER: &str = "{{";
const RIGHT_INTERPOLATION_IDENTIFIER: &str = "}}";

pub fn create_template_args_by_operator(
    id: usize,
    value: &Value,
    operator: &Option<Operator>,
    task_id_by_name: &HashMap<String, usize>,
) -> Value {
    let default_args = json!({});
    // generate template args dependant on operator type
    match operator {
        Some(Operator::BashOperator) => create_template_args_from_string(
            id,
            &serde_json::from_value::<TemplateBashTaskArgs>(value.clone())
                .expect("error parsing template bash args")
                .script,
            task_id_by_name,
        ),
        _ => value
            .as_object()
            .unwrap()
            .get("args")
            .unwrap_or(&default_args)
            .clone(),
    }
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct TemplateTaskArgs {
//     #[serde(default)]
//     pub args: Value,
// }

pub fn create_template_args_from_string(
    task_id: usize,
    original_string: &str,
    task_id_by_name: &HashMap<String, usize>,
) -> Value {
    assert!(!original_string.trim().is_empty());

    let mut temp_args = json!({ ORIGINAL_STRING_KEY: original_string });
    let mut temp_string = original_string.to_string();

    loop {
        let (left, right) = (
            temp_string.find(LEFT_INTERPOLATION_IDENTIFIER),
            temp_string.find(RIGHT_INTERPOLATION_IDENTIFIER),
        );

        if left.is_none() || right.is_none() {
            break;
        }
        let (left, right) = (left.unwrap(), right.unwrap());
        let chunks: Vec<&str> = temp_string[(left + 2)..(right)].trim().split('.').collect();

        let upstream_task_name = chunks[0];
        let upstream_id = task_id_by_name
            .get(upstream_task_name)
            .copied()
            .unwrap_or_else(|| get_id_by_task_name(upstream_task_name));

        let to_replace = &temp_string[left..(right + 2)].to_string();

        temp_args[to_replace] = json!({
            UPSTREAM_TASK_ID_KEY: upstream_id
        });

        if chunks.len() > 1 {
            temp_args[to_replace][UPSTREAM_TASK_RESULT_KEY] = chunks[1].into();
        }
        get_edges().write().unwrap().insert((upstream_id, task_id));
        temp_string.replace_range(left..(right + 2), "");
    }

    temp_args
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;
    use thepipelinetool_utils::{UPSTREAM_TASK_ID_KEY, UPSTREAM_TASK_RESULT_KEY};

    use crate::templating::create_template_args_from_string;

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
