#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms)]

mod account;
mod amount;
mod date;
mod directive;
mod error;
mod string;
mod transaction;

pub use crate::{
    account::Account,
    amount::{Amount, Expression},
    date::Date,
    directive::Directive,
    error::Error,
    transaction::{Posting, Transaction},
};

use directive::directive;
use nom::{bytes::complete::take_while, sequence::preceded, IResult};

pub struct Parser<'a> {
    rest: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            rest: content.trim(),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<(Date, Directive<'a>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }

        Some(match next(self.rest) {
            Ok((rest, directive)) => {
                self.rest = rest;
                Ok(directive)
            }
            Err(_) => {
                self.rest = "";
                Err(Error)
            }
        })
    }
}

fn next(input: &str) -> IResult<&str, (Date, Directive<'_>)> {
    preceded(take_while(|c: char| c.is_whitespace()), directive)(input)
}
