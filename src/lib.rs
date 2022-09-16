#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms, missing_docs, clippy::pedantic)]
#![cfg_attr(test, allow(clippy::needless_pass_by_value))]

//! A rust parsing library for [beancount](https://beancount.github.io/docs/) files
//!
//! At its core, this library provides is a [`Parser`] type that
//! is an iterator over the directives.
//!
//! ## Example
//! ```
//! use beancount_parser::{Date, Directive, Parser, Error};
//!
//! # fn main() -> Result<(), Error> {
//! let beancount = r#"
//! 2022-09-11 * "Coffee beans"
//!   Expenses:Groceries   10 CHF
//!   Assets:Bank
//! "#;
//!
//! let directives: Vec<(Date, Directive<'_>)> = Parser::new(beancount).collect::<Result<_, _>>()?;
//! assert_eq!(directives[0].1.as_transaction().unwrap().narration(), Some("Coffee beans"));
//!
//! let postings = directives[0].1.as_transaction().unwrap().postings();
//! assert_eq!(postings[0].amount().unwrap().currency(), "CHF");
//! assert_eq!(postings[0].amount().unwrap().value().try_into_f64()?, 10.0);
//! # Ok(()) }
//! ```

#[allow(missing_docs)]
mod account;
mod amount;
mod date;
#[allow(missing_docs)]
mod directive;
mod error;
mod string;
#[allow(missing_docs)]
mod transaction;

use crate::directive::directive;

pub use crate::{
    account::Account,
    amount::{Amount, ConversionError, Expression, Value},
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

/// Parser of a beancount document
///
/// It is an iterator over the beancount directives.
///
/// See the crate documentation for usage example.
pub struct Parser<'a> {
    rest: &'a str,
}

impl<'a> Parser<'a> {
    /// Create a new parser from the beancount string to parse
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
    alt((
        map(directive, |(date, directive)| directive.map(|d| (date, d))),
        value(None, comment_line),
    ))(input)
}
