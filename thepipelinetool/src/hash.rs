use std::hash::{DefaultHasher, Hash, Hasher};

use crate::dev::*;

pub fn hash_dag(nodes: &str, edges: &[(usize, usize)]) -> String {
    let mut hasher = DefaultHasher::new();
    let mut edges: Vec<&(usize, usize)> = edges.iter().collect();
    edges.sort();

    let to_hash = format!(
        "{}{}",
        serde_json::to_string(&nodes).unwrap(),
        &serde_json::to_string(&edges).unwrap()
    );
    to_hash.hash(&mut hasher);
    to_base62(hasher.finish())
}
