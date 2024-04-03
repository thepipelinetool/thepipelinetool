use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
};

use thepipelinetool_core::dev::*;

pub fn display_hash(tasks: &[Task], edges: &HashSet<(usize, usize)>) {
    let hash = hash_dag(
        &serde_json::to_string(tasks).unwrap(),
        &edges.iter().copied().collect::<Vec<(usize, usize)>>(),
    );
    print!("{hash}");
}

fn hash_dag(nodes: &str, edges: &[(usize, usize)]) -> String {
    let mut hasher = DefaultHasher::new();
    let mut edges = edges.to_vec();
    edges.sort();

    format!(
        "{}{}",
        serde_json::to_string(&nodes).unwrap(),
        &serde_json::to_string(&edges).unwrap()
    )
    .hash(&mut hasher);
    to_base62(hasher.finish())
}

const BASE62: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

fn to_base62(mut num: u64) -> String {
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
