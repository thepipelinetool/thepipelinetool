use std::{marker::PhantomData, ops::{BitOr, Shl, Shr}};

use serde::Serialize;
use serde_json::Value;
use thepipelinetool_task::task_ref_inner::TaskRefInner;

use crate::{flow::{par, seq}, TaskRef};


impl<T, G> Shr<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: TaskRef<G>) -> Self::Output {
        seq(&self, &rhs)
    }
}

impl<T, G> Shr<&TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(self, rhs)
    }
}

impl<T, G> Shr<TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: TaskRef<G>) -> Self::Output {
        seq(self, &rhs)
    }
}

impl<T, G> Shr<&TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn shr(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(&self, rhs)
    }
}

impl<T, G> Shl<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: TaskRef<G>) -> Self::Output {
        seq(&rhs, &self)
    }
}

impl<T, G> Shl<&TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(rhs, self)
    }
}

impl<T, G> Shl<TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: TaskRef<G>) -> Self::Output {
        seq(&rhs, self)
    }
}

impl<T, G> Shl<&TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<T>;
    fn shl(self, rhs: &TaskRef<G>) -> Self::Output {
        seq(rhs, &self)
    }
}

impl<T, G> BitOr<TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: TaskRef<G>) -> Self::Output {
        par(&self, &rhs)
    }
}

impl<T, G> BitOr<&TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: &TaskRef<G>) -> Self::Output {
        par(self, rhs)
    }
}

impl<T, G> BitOr<TaskRef<G>> for &TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: TaskRef<G>) -> Self::Output {
        par(self, &rhs)
    }
}

impl<T, G> BitOr<&TaskRef<G>> for TaskRef<T>
where
    T: Serialize,
    G: Serialize,
{
    type Output = TaskRef<G>;
    fn bitor(self, rhs: &TaskRef<G>) -> Self::Output {
        par(&self, rhs)
    }
}

impl<T: Serialize> Serialize for TaskRef<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T: Serialize> TaskRef<T> {
    pub fn get(&self, key: &str) -> TaskRef<Value> {
        assert!(self.0.task_ids.len() == 1, "Cannot use parallel ref as arg");

        TaskRef(TaskRefInner {
            task_ids: self.0.task_ids.clone(),
            key: Some(key.to_string()),
            _marker: PhantomData,
        })
    }

    pub fn value(&self) -> TaskRef<Value> {
        assert!(self.0.task_ids.len() == 1, "Cannot use parallel ref as arg");

        TaskRef(TaskRefInner {
            task_ids: self.0.task_ids.clone(),
            key: None,
            _marker: PhantomData,
        })
    }
}