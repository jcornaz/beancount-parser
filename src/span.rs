#![cfg(all(test, feature = "unstable"))]

#[derive(Debug)]
pub struct Spanned<T> {
    inner: T,
}

impl<T> Spanned<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Spanned<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}
