use std::{
    fs::File,
    io::{BufRead, BufReader, Error, Read, Write},
    path::{Path, PathBuf},
    process::{self, Command, ExitStatus, Stdio},
    thread,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const UPSTREAM_TASK_ID_KEY: &str = "upstream_task_id";
pub const UPSTREAM_TASK_RESULT_KEY: &str = "key";

pub fn function_name_as_string<T>(_: T) -> String {
    let name = std::any::type_name::<T>();
    let name = &name.replace(['}', '{'], "");

    // Find and cut the rest of the path
    match name[..name.len()].rfind(':') {
        Some(pos) => name[pos + 1..name.len()].into(),
        None => name[..name.len()].into(),
    }
}

pub fn value_from_file<F: for<'a> Deserialize<'a>>(file_path: &Path) -> Result<F, Error> {
    let mut file = File::open(file_path)?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)?;
    Ok(serde_json::from_str(&json_data)?)
}

pub fn value_to_file<F: Serialize>(v: &F, file_path: &Path) {
    let json_string = serde_json::to_string_pretty(v).unwrap();
    let mut file =
        File::create(file_path).unwrap_or_else(|e| panic!("couldn't write to file\n {e}"));

    file.write_all(json_string.as_bytes()).unwrap();
}

pub fn execute_function_using_json_files(
    in_file: &Path,
    out_file: &Path,
    task_function: &dyn Fn(Value) -> Value,
) {
    let task_args = value_from_file(in_file).unwrap(); // TODO handle error
    let task_result = (task_function)(task_args);
    value_to_file(&task_result, out_file);
    process::exit(0);
}

pub fn execute_function_using_json_str_args(
    task_args_str: &str,
    task_function: &dyn Fn(Value) -> Value,
) {
    let task_args = serde_json::from_str(task_args_str).unwrap();
    let task_result = (task_function)(task_args);
    println!("{}", serde_json::to_string(&task_result).unwrap());
    process::exit(0);
}

pub fn collector(args: Value) -> Value {
    args
}

const BASE62: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

pub fn to_base62(mut num: u64) -> String {
    let mut chars = vec![];

    while num > 0 {
        chars.push(BASE62[(num % 62) as usize]);
        num /= 62;
    }

    chars.reverse();

    while chars.len() < 7 {
        chars.push('0');
    }

    chars.truncate(7); // Ensure length is 7
    chars.iter().collect()
}

pub fn spawn(
    mut cmd: Command,
    handle_stdout_log: Box<dyn Fn(String) + Send>,
    handle_stderr_log: Box<dyn Fn(String) + Send>,
) -> (ExitStatus, bool) {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start command");

    let stdout = child.stdout.take().expect("failed to take stdout");
    let stderr = child.stderr.take().expect("failed to take stderr");

    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = format!("{}\n", line.expect("failed to read line from stdout"));
            handle_stdout_log(line);
        }
    });

    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = format!("{}\n", line.expect("failed to read line from stdout"));
            handle_stderr_log(line);
        }
    });

    let status = child.wait().expect("failed to wait on child");
    let timed_out = matches!(status.code(), Some(124));
    stdout_handle.join().expect("stdout thread panicked");
    stderr_handle.join().expect("stderr thread panicked");

    (status, timed_out)
}

#[derive(PartialEq)]
pub enum DagType {
    Binary,
    YAML,
}

pub fn get_dag_type_by_path(path: PathBuf) -> DagType {
    if let Some(ext) = path.extension() {
        match ext.to_str().unwrap() {
            "yaml" => {
                return DagType::YAML;
            }
            _ => {}
        }
    }
    DagType::Binary
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::DagType;

    use super::get_dag_type_by_path;

    #[test]
    fn test_get_dag_type_by_path() {
        assert!(get_dag_type_by_path(Path::new("hello").to_path_buf()) == DagType::Binary);
        assert!(get_dag_type_by_path(Path::new("hello.yaml").to_path_buf()) == DagType::YAML);
    }
}

pub fn run_bash_commmand(args: &Vec<&str>, silent: bool) -> Value {
    let output = Command::new(args[0].to_string())
        .args(&mut (args.clone()[1..]))
        .output()
        .unwrap_or_else(|_| panic!("failed to run command:\n{}\n\n", args.join(" ")));
    let result_raw = String::from_utf8_lossy(&output.stdout);
    let err_raw = String::from_utf8_lossy(&output.stderr);

    if !silent {
        print!("{}", result_raw);
    }
    if !output.status.success() {
        eprint!("{}", err_raw);
        panic!("failed to run command:\n{}\n\n", args.join(" "));
    }

    json!(result_raw.to_string().trim_end())
}
