use std::collections::HashSet;

use serde::Serialize;

use crate::{prelude::*, statics::*};

pub fn seq<T: Serialize, G: Serialize>(a: &TaskRef<T>, b: &TaskRef<G>) -> TaskRef<G> {
    let mut last: usize = 0;
    let mut edges = get_edges().write().unwrap();

    for up in a.0.task_ids.iter() {
        for down in b.0.task_ids.iter() {
            edges.insert((*up, *down));
            last = *down;
        }
    }

    TaskRef(TaskRefInner {
        task_ids: HashSet::from([last]),
        key: None,
        _marker: std::marker::PhantomData,
    })
}

pub fn par<T: Serialize, G: Serialize>(a: &TaskRef<T>, b: &TaskRef<G>) -> TaskRef<G> {
    let mut task_ids: HashSet<usize> = HashSet::new();
    task_ids.extend(&a.0.task_ids);
    task_ids.extend(&b.0.task_ids);

    TaskRef(TaskRefInner {
        task_ids,
        key: None,
        _marker: std::marker::PhantomData,
    })
}
