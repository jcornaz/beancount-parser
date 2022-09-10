#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms)]
#![allow(dead_code)]

mod account;
mod amount;
mod date;
mod directive;
mod error;
mod string;
mod transaction;

pub use account::Account;
pub use date::Date;
pub use directive::Directive;
pub use error::Error;
pub use transaction::{Posting, Transaction};

pub struct Parser<'a> {
    rest: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { rest: content }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<(Date, Directive<'a>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
