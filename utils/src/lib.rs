use std::{
    fs::File,
    io::{Read, Write},
    process,
};

use serde_json::Value;

pub fn function_name_as_string<T>(_: T) -> String {
    let name = std::any::type_name::<T>();
    let name = &name.replace(['}', '{'], "");

    // Find and cut the rest of the path
    match name[..name.len()].rfind(':') {
        Some(pos) => name[pos + 1..name.len()].into(),
        None => name[..name.len()].into(),
    }
}

pub fn value_from_file(file_path: &str) -> Value {
    let mut file = File::open(file_path)
        .unwrap_or_else(|_| panic!("could not read file_path: {}", &file_path));
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();
    serde_json::from_str(&json_data).unwrap()
}

pub fn value_to_file(v: &Value, file_path: &str) {
    let json_string = serde_json::to_string_pretty(v).unwrap();
    let mut file =
        File::create(file_path).unwrap_or_else(|_| panic!("couldn't write to file {file_path}"));

    file.write_all(json_string.as_bytes()).unwrap();
}

pub fn execute_function(in_file: &str, out_file: &str, task_function: &dyn Fn(Value) -> Value) {
    let task_args = value_from_file(in_file);
    let task_result = (task_function)(task_args);

    value_to_file(&task_result, out_file);
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
