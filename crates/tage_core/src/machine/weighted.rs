#[derive(Clone, Debug)]
pub struct Weighted<T> {
    pub weight: i32,
    pub content: T,
}

impl<T> Weighted<T> {
    pub fn new(weight: i32, content: T) -> Weighted<T> {
        Self { weight, content }
    }
}
