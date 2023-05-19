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
#![allow(clippy::needless_pass_by_value, clippy::deprecated_semver)]
#![cfg_attr(
    not(test),
    warn(
        missing_debug_implementations,
        clippy::get_unwrap,
        clippy::unwrap_in_result,
        clippy::unwrap_used
    )
)]
#![cfg_attr(feature = "unstable", allow(missing_docs, clippy::missing_errors_doc))]
#![cfg_attr(has_doc_auto_cfg, feature(doc_auto_cfg))]

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

pub mod account;
pub mod amount;
mod assertion;
mod close;
mod commodity;
mod date;
mod directive;
mod error;
mod event;
mod include;
#[cfg(feature = "unstable")]
pub mod metadata;
#[cfg(not(feature = "unstable"))]
mod metadata;
mod nom_parser;
mod open;
mod option;
mod pad;
#[cfg(feature = "unstable")]
#[doc(hidden)]
pub mod pest_parser;
mod price;
mod string;
pub mod transaction;

/// Type of account
#[deprecated(
    since = "1.15.0",
    note = "Use `AccountType` or `account::Type` instead"
)]
#[doc(hidden)]
pub type Type = account::Type;

pub use crate::include::Include;
pub use crate::nom_parser::Parser;
pub use crate::{
    account::{Account, Type as AccountType},
    amount::Amount,
    assertion::Assertion,
    close::Close,
    date::Date,
    directive::Directive,
    error::Error,
    open::Open,
    pad::Pad,
    price::Price,
    transaction::Transaction,
};

#[cfg(feature = "unstable")]
pub use crate::{commodity::Commodity, event::Event, option::Option};

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;
