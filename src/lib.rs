#![deny(future_incompatible, unsafe_code)]
#![warn(nonstandard_style, rust_2018_idioms, missing_docs, clippy::pedantic)]
#![cfg_attr(test, allow(clippy::needless_pass_by_value))]
#![cfg_attr(nightly, feature(doc_auto_cfg))]

//! A rust parsing library for [beancount](https://beancount.github.io/docs/) files
//!
//! At its core, this library provides is a [`Parser`] type that
//! is an iterator over the directives.
//!
//! ## Example
//! ```
//! use beancount_parser::{Directive, Parser, Error};
//!
//! # fn main() -> Result<(), Error> {
//! let beancount = r#"
//! 2022-09-11 * "Coffee beans"
//!   Expenses:Groceries   10 CHF
//!   Assets:Bank
//! "#;
//!
//! let directives: Vec<Directive<'_>> = Parser::new(beancount).collect::<Result<_, _>>()?;
//! let transaction = directives[0].as_transaction().unwrap();
//! assert_eq!(transaction.narration(), Some("Coffee beans"));
//!
//! let first_posting_amount = transaction.postings()[0].amount().unwrap();
//! assert_eq!(first_posting_amount.currency(), "CHF");
//! assert_eq!(first_posting_amount.value().try_into_f64()?, 10.0);
//! # Ok(()) }
//! ```

pub mod account;
pub mod amount;
mod close;
mod date;
mod directive;
mod error;
mod open;
mod price;
mod string;
pub mod transaction;

use crate::directive::directive;

pub use crate::{
    account::Account, amount::Amount, close::Close, date::Date, directive::Directive, error::Error,
    open::Open, price::Price, transaction::Transaction,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::{line_ending, not_line_ending},
        streaming::space1,
    },
    combinator::{map, opt, value},
    sequence::{preceded, tuple},
    IResult,
};

/// Parser of a beancount document
///
/// It is an iterator over the beancount directives.
///
/// See the crate documentation for usage example.
pub struct Parser<'a> {
    rest: &'a str,
    tags: Vec<&'a str>,
    line: u64,
}

impl<'a> Parser<'a> {
    /// Create a new parser from the beancount string to parse
    #[must_use]
    pub fn new(content: &'a str) -> Self {
        Self {
            rest: content,
            tags: Vec::new(),
            line: 1,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Directive<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.rest.is_empty() {
            if let Ok((rest, chunk)) = chunk(self.rest) {
                self.line += 1;
                self.rest = rest;
                match chunk {
                    Chunk::Directive(mut directive) => {
                        if let Directive::Transaction(trx) = &mut directive {
                            self.line += trx.postings().len() as u64;
                            trx.tags.extend(&self.tags);
                        }
                        return Some(Ok(directive));
                    }
                    Chunk::PushTag(tag) => self.tags.push(tag),
                    Chunk::PopTag(tag) => self.tags.retain(|&t| t != tag),
                    Chunk::Comment => (),
                }
            } else {
                self.rest = "";
                return Some(Err(Error::from_parsing(self.line)));
            }
        }
        None
    }
}

fn chunk(input: &str) -> IResult<&str, Chunk<'_>> {
    alt((
        map(directive, Chunk::Directive),
        map(pushtag, Chunk::PushTag),
        map(poptag, Chunk::PopTag),
        value(Chunk::Comment, tuple((not_line_ending, opt(line_ending)))),
    ))(input)
}

fn pushtag(input: &str) -> IResult<&str, &str> {
    preceded(tuple((tag("pushtag"), space1)), transaction::tag)(input)
}

fn poptag(input: &str) -> IResult<&str, &str> {
    preceded(tuple((tag("poptag"), space1)), transaction::tag)(input)
}

#[derive(Debug, Clone)]
enum Chunk<'a> {
    Directive(Directive<'a>),
    Comment,
    PushTag(&'a str),
    PopTag(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushtag() {
        let input = "pushtag #test";
        let (_, chunk) = chunk(input).expect("should successfully parse the input");
        assert!(matches!(chunk, Chunk::PushTag("test")));
    }

    #[test]
    fn poptag() {
        let input = "poptag #test";
        let (_, chunk) = chunk(input).expect("should successfully parse the input");
        assert!(matches!(chunk, Chunk::PopTag("test")));
    }
}
