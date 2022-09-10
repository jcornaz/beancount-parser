#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms, clippy::pedantic)]

mod account;
mod amount;
mod date;
mod directive;
mod error;
mod string;
mod transaction;

use crate::directive::directive;
pub use crate::{
    account::Account,
    amount::{Amount, Expression},
    date::Date,
    directive::Directive,
    error::Error,
    transaction::{Posting, Transaction},
};

use nom::{
    branch::alt,
    combinator::{map, value},
    IResult,
};
use string::comment_line;

pub struct Parser<'a> {
    rest: &'a str,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(content: &'a str) -> Self {
        Self { rest: content }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<(Date, Directive<'a>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.rest.is_empty() {
            if let Ok((rest, directive)) = next(self.rest) {
                self.rest = rest;
                if let Some(directive) = directive {
                    return Some(Ok(directive));
                }
            } else {
                self.rest = "";
                return Some(Err(Error));
            }
        }
        None
    }
}

fn next(input: &str) -> IResult<&str, Option<(Date, Directive<'_>)>> {
    alt((map(directive, Some), value(None, comment_line)))(input)
}
