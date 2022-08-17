#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms)]
#![allow(dead_code)]

use rust_decimal::Decimal;

pub use date::Date;

mod date;
mod parser;

#[derive(Debug, Clone)]
pub struct Amount<'a> {
    value: Value,
    currency: &'a str,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Value(Decimal);
