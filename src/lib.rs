#![deny(future_incompatible, nonstandard_style, unsafe_code, private_in_public)]
#![warn(rust_2018_idioms, clippy::pedantic, missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! A parsing library for the [beancount language](https://beancount.github.io/docs/beancount_language_syntax.html)
//!
//! # Usage
//!
//! Use [`parse`] to get an instance of [`BeancountFile`].
//!
//! This is generic over the decimal type. The examples use `f64` as a decimal type.
//! You may also use `Decimal` from the [rust_decimal crate].
//!
//! [rust_decimal crate]: https://docs.rs/rust_decimal
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
    amount::{Amount, Currency, Decimal, Price},
    date::Date,
    error::Error,
    event::Event,
    metadata::Value as MetadataValue,
    transaction::{Cost, Flag, Posting, PostingPrice, Transaction},
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
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

/// Parse the input beancount file and return an instance of [`BeancountFile`] on success
///
/// It is generic over the [`Decimal`] type `D`.
///
/// See the root crate documentation for an example.
///
/// # Errors
///
/// Returns an [`Error`] in case of invalid beancount syntax found.
pub fn parse<D: Decimal>(input: &str) -> Result<BeancountFile<'_, D>, Error<'_>> {
    match all_consuming(beancount_file)(Span::new(input)).finish() {
        Ok((_, content)) => Ok(content),
        Err(nom::error::Error { input, .. }) => Err(Error::new(input)),
    }
}

/// Main struct representing a parsed beancount file.
///
/// To get an instance of this, use [`parse`].
///
/// For an example, look at the root crate documentation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BeancountFile<'a, D> {
    /// Map of options declared in the file
    ///
    /// See: <https://beancount.github.io/docs/beancount_language_syntax.html#options>
    options: HashMap<&'a str, &'a str>,
    /// Pathes of include directives
    ///
    /// See: <https://beancount.github.io/docs/beancount_language_syntax.html#includes>
    includes: HashSet<&'a Path>,
    /// List of [`Directive`] found in the file
    pub directives: Vec<Directive<'a, D>>,
}

impl<'a, D> BeancountFile<'a, D> {
    /// Returns the value for of the option if defined
    ///
    /// See: <https://beancount.github.io/docs/beancount_language_syntax.html#options>
    #[must_use]
    pub fn option(&self, key: &str) -> Option<&'a str> {
        self.options.get(key).copied()
    }
}

impl<'a, D> BeancountFile<'a, D> {
    /// Return an iterator over the include directives
    ///
    /// See: <https://beancount.github.io/docs/beancount_language_syntax.html#includes>
    pub fn includes(&self) -> impl Iterator<Item = &'a Path> + '_ {
        self.includes.iter().copied()
    }
}

/// A beancount "directive"
///
/// It has fields common to all directives, and a [`Directive::content`] field with
/// a different content for each directive type.
///
/// ```
/// # use beancount_parser_2::{BeancountFile, DirectiveContent};
/// let input = r#"
/// 2022-01-01 open Assets:Cash
/// 2022-01-01 * "Grocery shopping"
///   Expensses:Groceerices  10 CHF
///   Assets:Cash
/// "#;
/// let beancount: BeancountFile<f64> = beancount_parser_2::parse(input).unwrap();
/// assert_eq!(beancount.directives.len(), 2);
/// for directive in beancount.directives {
///    println!("line: {}", directive.line_number);
///    println!("metadata: {:#?}", directive.metadata);
///    match directive.content {
///       DirectiveContent::Open(open) => println!("open account directive: {open:?}"),
///       DirectiveContent::Transaction(trx) => println!("transaction: {trx:?}"),
///       other => println!("unknown directive: {other:?}"),
///    }
/// }
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Directive<'a, D> {
    /// Date of the directive
    pub date: Date,
    /// Content of the directive that is specific to each directive type
    pub content: DirectiveContent<'a, D>,
    /// Metadata associated to the directive
    ///
    /// See: <https://beancount.github.io/docs/beancount_language_syntax.html#metadata>
    pub metadata: HashMap<&'a str, metadata::Value<'a, D>>,
    /// Line number where the directive was found in the input file
    pub line_number: u32,
}

/// Directive specific content
#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DirectiveContent<'a, D> {
    Transaction(transaction::Transaction<'a, D>),
    Price(amount::Price<'a, D>),
    Balance(account::Balance<'a, D>),
    Open(account::Open<'a>),
    Close(account::Close<'a>),
    Pad(account::Pad<'a>),
    Commodity(Currency<'a>),
    Event(event::Event<'a>),
}

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;

fn beancount_file<D: Decimal>(input: Span<'_>) -> IResult<'_, BeancountFile<'_, D>> {
    let mut iter = iterator(input, entry);
    let mut options = HashMap::new();
    let mut includes = HashSet::new();
    let mut tag_stack = HashSet::new();
    let mut directives = Vec::new();
    for entry in &mut iter {
        match entry {
            Entry::Directive(mut d) => {
                if let DirectiveContent::Transaction(trx) = &mut d.content {
                    trx.tags.extend(tag_stack.iter());
                }
                directives.push(d);
            }
            Entry::Option { key, value } => {
                options.insert(key, value);
            }
            Entry::Include(path) => {
                includes.insert(path);
            }
            Entry::PushTag(tag) => {
                tag_stack.insert(tag);
            }
            Entry::PopTag(tag) => {
                tag_stack.remove(tag);
            }
            Entry::Comment => (),
        }
    }
    let (input, _) = iter.finish()?;
    Ok((
        input,
        BeancountFile {
            options,
            includes,
            directives,
        },
    ))
}

enum Entry<'a, D> {
    Directive(Directive<'a, D>),
    Option { key: &'a str, value: &'a str },
    Include(&'a Path),
    PushTag(&'a str),
    PopTag(&'a str),
    Comment,
}

fn entry<D: Decimal>(input: Span<'_>) -> IResult<'_, Entry<'_, D>> {
    alt((
        directive.map(Entry::Directive),
        option.map(|(key, value)| Entry::Option { key, value }),
        include.map(|p| Entry::Include(p)),
        tag_stack_operation,
        line.map(|_| Entry::Comment),
    ))(input)
}

fn directive<D: Decimal>(input: Span<'_>) -> IResult<'_, Directive<'_, D>> {
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
    let (input, _) = end_of_line(input)?;
    Ok((input, (key, value)))
}

fn include(input: Span<'_>) -> IResult<'_, &Path> {
    let (input, _) = tag("include")(input)?;
    let (input, path) = cut(delimited(space1, string, end_of_line))(input)?;
    Ok((input, Path::new(path)))
}

fn tag_stack_operation<D>(input: Span<'_>) -> IResult<'_, Entry<'_, D>> {
    alt((
        preceded(tuple((tag("pushtag"), space1)), transaction::parse_tag).map(Entry::PushTag),
        preceded(tuple((tag("poptag"), space1)), transaction::parse_tag).map(Entry::PopTag),
    ))(input)
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
