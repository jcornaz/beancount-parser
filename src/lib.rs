#![deny(
    future_incompatible,
    nonstandard_style,
    unsafe_code,
    private_in_public,
    missing_docs,
    unused_results,
    missing_docs
)]
#![warn(rust_2018_idioms, clippy::pedantic)]
#![cfg_attr(test, allow(clippy::needless_pass_by_value))]
#![cfg_attr(
    not(test),
    warn(
        missing_debug_implementations,
        clippy::get_unwrap,
        clippy::unwrap_in_result,
        clippy::unwrap_used
    )
)]
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

#[cfg(test)]
#[macro_use]
extern crate rstest;

pub use crate::nom_parser::Parser;

pub use crate::include::Include;
pub use crate::{
    account::Account, amount::Amount, assertion::Assertion, close::Close, date::Date,
    directive::Directive, error::Error, open::Open, pad::Pad, price::Price,
    transaction::Transaction,
};

pub mod account;
pub mod amount;
mod assertion;
mod close;
mod date;
mod directive;
mod error;
mod include;
mod metadata;
mod nom_parser;
mod open;
mod pad;
mod pest_parser;
mod price;
mod string;
pub mod transaction;
