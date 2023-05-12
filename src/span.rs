#![cfg(feature = "unstable")]
#![allow(unused)]

#[derive(Debug)]
pub struct Spanned<T> {
    span: Span,
    inner: T,
}

#[derive(Debug)]
struct Span {
    from: Location,
    to: Location,
}

#[derive(Debug)]
struct Location {
    line: usize,
    col: usize,
}

impl<T> Spanned<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}
