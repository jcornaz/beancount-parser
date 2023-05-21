#![deny(future_incompatible, nonstandard_style, unsafe_code, private_in_public)]
#![warn(rust_2018_idioms, clippy::pedantic)]

//! A parsing library for the [beancount language](https://beancount.github.io/docs/beancount_language_syntax.htm)
//!
//! # Usage
//!
//! Use [`parse`] to get an instance of [`BeancountFile`].
//!
//! Note that it is generic over the decimal type.
//! `Decimal` from the crate [rust_decimal](https://docs.rs/rust_decimal) is a solid choice, but it also works with `f64`
//! or any other decimal type that implement `FromStr`.
//!
//! ```
//! use beancount_parser_2::{BeancountFile, DirectiveContent};
//!
//! # fn main() -> Result<(), beancount_parser_2::Error<'static>> {
//! let input = r#"
//! 2023-05-20 * "Coffee beans"
//!   Expenses:Groceries   10 CHF
//!   Assets:Checking
//! "#;
//!
//! // Parse into the `BeancountFile` struct:
//! let beancount: BeancountFile<f64> = beancount_parser_2::parse::<f64>(input)?;
//!
//! let directive = &beancount.directives[0];
//! assert_eq!(directive.date.year, 2023);
//! assert_eq!(directive.date.month, 5);
//! assert_eq!(directive.date.day, 20);
//!
//! let DirectiveContent::Transaction(trx) = &directive.content else {
//!     panic!("was not a transaction")
//! };
//! assert_eq!(trx.narration, Some("Coffee beans"));
//! assert_eq!(trx.postings[0].account.as_str(), "Expenses:Groceries");
//! assert_eq!(trx.postings[0].amount.unwrap().value, 10.0);
//! assert_eq!(trx.postings[0].amount.unwrap().currency.as_str(), "CHF");
//! assert_eq!(trx.postings[1].account.as_str(), "Assets:Checking");
//! assert_eq!(trx.postings[1].amount, None);
//! # Ok(()) }
//! ```

mod account;
mod amount;
mod date;
mod error;
mod event;
mod metadata;
mod transaction;

pub use crate::{
    account::{Account, Balance, Close, Open, Pad},
    amount::{Amount, Currency, Price},
    date::Date,
    error::Error,
    event::Event,
    metadata::Value as MetadataValue,
    transaction::{Flag, Posting, Transaction},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{char, line_ending, not_line_ending, space0, space1},
    combinator::{all_consuming, cut, eof, iterator, map, opt},
    sequence::{delimited, preceded, terminated, tuple},
    Finish, Parser,
};
use nom_locate::position;
use std::{collections::HashMap, str::FromStr};

/// Parse the input beancount file and return an instance of [`BeancountFile`] on success
///
/// Is is generic over the decimal type `D`, which can be anytype implementing `FromStr`.
///
/// See the root crate documentation for an example.
///
/// # Errors
///
/// Returns an [`Error`] in case of invalid beancount syntax found.
pub fn parse<D: FromStr>(input: &str) -> Result<BeancountFile<'_, D>, Error<'_>> {
    match all_consuming(beancount_file)(Span::new(input)).finish() {
        Ok((_, content)) => Ok(content),
        Err(nom::error::Error { input, .. }) => Err(Error::new(input)),
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct BeancountFile<'a, D> {
    pub options: HashMap<&'a str, &'a str>,
    pub directives: Vec<Directive<'a, D>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Directive<'a, D> {
    pub date: Date,
    pub content: DirectiveContent<'a, D>,
    pub metadata: HashMap<&'a str, metadata::Value<'a>>,
    pub line_number: u32,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DirectiveContent<'a, D> {
    Transaction(transaction::Transaction<'a, D>),
    Price(amount::Price<'a, D>),
    Balance(account::Balance<'a, D>),
    Open(account::Open<'a>),
    Close(account::Close<'a>),
    /// See [`Pad`]
    Pad(account::Pad<'a>),
    Commodity(Currency<'a>),
    Event(event::Event<'a>),
}

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;

fn beancount_file<D: FromStr>(input: Span<'_>) -> IResult<'_, BeancountFile<'_, D>> {
    let mut iter = iterator(input, entry);
    let mut options = HashMap::new();
    let mut directives = Vec::new();
    for entry in &mut iter {
        match entry {
            Entry::Directive(d) => {
                directives.push(d);
            }
            Entry::Option { key, value } => {
                options.insert(key, value);
            }
            Entry::Comment => (),
        }
    }
    let (input, _) = iter.finish()?;
    Ok((
        input,
        BeancountFile {
            options,
            directives,
        },
    ))
}

enum Entry<'a, D> {
    Directive(Directive<'a, D>),
    Option { key: &'a str, value: &'a str },
    Comment,
}

fn entry<D: FromStr>(input: Span<'_>) -> IResult<'_, Entry<'_, D>> {
    alt((
        directive.map(Entry::Directive),
        line.map(|_| Entry::Comment),
        option.map(|(key, value)| Entry::Option { key, value }),
    ))(input)
}

fn directive<D: FromStr>(input: Span<'_>) -> IResult<'_, Directive<'_, D>> {
    let (input, position) = position(input)?;
    let (input, date) = date::parse(input)?;
    let (input, _) = cut(space1)(input)?;
    let (input, (content, metadata)) = alt((
        map(transaction::parse, |(t, m)| {
            (DirectiveContent::Transaction(t), m)
        }),
        tuple((
            terminated(
                alt((
                    map(
                        preceded(tag("price"), cut(preceded(space1, amount::price))),
                        DirectiveContent::Price,
                    ),
                    map(
                        preceded(tag("balance"), cut(preceded(space1, account::balance))),
                        DirectiveContent::Balance,
                    ),
                    map(
                        preceded(tag("open"), cut(preceded(space1, account::open))),
                        DirectiveContent::Open,
                    ),
                    map(
                        preceded(tag("close"), cut(preceded(space1, account::close))),
                        DirectiveContent::Close,
                    ),
                    map(
                        preceded(tag("pad"), cut(preceded(space1, account::pad))),
                        DirectiveContent::Pad,
                    ),
                    map(
                        preceded(tag("commodity"), cut(preceded(space1, amount::currency))),
                        DirectiveContent::Commodity,
                    ),
                    map(
                        preceded(tag("event"), cut(preceded(space1, event::parse))),
                        DirectiveContent::Event,
                    ),
                )),
                end_of_line,
            ),
            metadata::parse,
        )),
    ))(input)?;
    Ok((
        input,
        Directive {
            date,
            content,
            metadata,
            line_number: position.location_line(),
        },
    ))
}

fn option(input: Span<'_>) -> IResult<'_, (&str, &str)> {
    let (input, _) = tag("option")(input)?;
    let (input, key) = preceded(space1, string)(input)?;
    let (input, value) = preceded(space1, string)(input)?;
    Ok((input, (key, value)))
}

fn end_of_line(input: Span<'_>) -> IResult<'_, ()> {
    let (input, _) = space0(input)?;
    let (input, _) = opt(comment)(input)?;
    let (input, _) = alt((line_ending, eof))(input)?;
    Ok((input, ()))
}

fn comment(input: Span<'_>) -> IResult<'_, ()> {
    let (input, _) = char(';')(input)?;
    let (input, _) = not_line_ending(input)?;
    Ok((input, ()))
}

fn line(input: Span<'_>) -> IResult<'_, ()> {
    let (input, _) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

fn string(input: Span<'_>) -> IResult<'_, &str> {
    map(
        delimited(char('"'), take_till(|c: char| c == '"'), char('"')),
        |s: Span<'_>| *s.fragment(),
    )(input)
}
