use serde::Serialize;

#[derive(Serialize)]
pub enum Branch<T: Serialize> {
    Left(T),
    Right(T),
}
