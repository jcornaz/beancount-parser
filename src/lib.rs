#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms)]
#![allow(dead_code)]

use rust_decimal::Decimal;

mod parser;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Date {
    year: u16,
    month_of_year: u8,
    day_of_month: u8,
}

#[derive(Debug, Clone)]
pub struct Amount<'a> {
    value: Value,
    currency: &'a str,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Value(Decimal);
