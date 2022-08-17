#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms)]
#![allow(dead_code)]

pub use date::Date;

mod date;
mod expression;
