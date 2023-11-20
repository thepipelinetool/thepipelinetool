use serde::Serialize;

#[derive(Serialize)]
pub struct Branch<T: Serialize> {
    is_left: bool,
    val: T,
}

impl<T: Serialize> Branch<T> {
    pub fn left(val: T) -> Self {
        Self { is_left: true, val }
    }

    pub fn right(val: T) -> Self {
        Self { is_left: false, val }
    }
}
