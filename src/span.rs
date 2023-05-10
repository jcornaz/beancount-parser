#![cfg(feature = "unstable-span")]
#![allow(unused)]

pub struct Spanned<T> {
    span: Span,
    data: T,
}

struct Span {
    from: Location,
    to: Location,
}

struct Location {
    line: usize,
    col: usize,
}
