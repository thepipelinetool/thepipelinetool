use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskResult {
    pub task_id: usize,
    pub result: Value,
    pub attempt: usize,
    pub max_attempts: isize,
    pub function_name: String,
    pub success: bool,
    pub resolved_args_str: String,
    pub started: String,
    pub ended: String,
    pub elapsed: i64,
    pub premature_failure: bool,
    pub premature_failure_error_str: String,
    pub is_branch: bool,
}

impl TaskResult {
    pub fn needs_retry(&self) -> bool {
        !self.premature_failure
            && !self.success
            && (self.max_attempts == -1 || self.attempt < self.max_attempts as usize)
    }

    pub fn premature_error(
        task_id: usize,
        attempt: usize,
        max_attempts: isize,
        function_name: String,
        premature_failure_error_str: String,
        is_branch: bool,
    ) -> Self {
        let start = Utc::now();

        Self {
            task_id,
            result: Value::Null,
            attempt,
            max_attempts,
            function_name,
            success: false,
            resolved_args_str: "".into(),
            started: start.to_rfc3339(),
            ended: start.to_rfc3339(),
            elapsed: 0,
            premature_failure: true,
            premature_failure_error_str,
            is_branch,
        }
    }

    pub fn print_task_result(&self, template_args: Value, log: String) {
        println!("=============================================");
        println!("TASK RUN");
        println!("id:\t\t{}", self.task_id);
        println!("attempt:\t{}/{}", self.attempt, self.max_attempts);
        println!("function_name:\t{}", self.function_name);
        println!(
            "template_args:\t{}",
            serde_json::to_string_pretty(&template_args).unwrap()
        );
        println!("rendered_args:\t{}", self.resolved_args_str);
        println!(
            "result: {}",
            serde_json::to_string_pretty(&self.result).unwrap()
        );
        println!("------Log------\n{}\n------------------", log);
        println!("success:\t{}", self.success);
        println!("started:\t{}", self.started);
        println!("ended:\t\t{}", self.ended);
        println!("time_elapsed:\t{}s", self.elapsed);

        if !self.success {
            println!("premature_failure: {}", self.premature_failure);
            if self.premature_failure {
                println!(
                    "premature_failure_error: {}",
                    self.premature_failure_error_str
                );
            }
        }

        println!("=============================================");
    }
}
