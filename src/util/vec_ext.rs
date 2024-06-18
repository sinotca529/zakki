pub trait VecExt {
    fn extended(self, other: Self) -> Self;
}

impl<T> VecExt for Vec<T> {
    fn extended(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}
