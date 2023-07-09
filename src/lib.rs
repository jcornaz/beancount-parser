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
//! use beancount_parser::{BeancountFile, DirectiveContent};
//!
//! # fn main() -> Result<(), beancount_parser::Error> {
//! let input = r#"
//! 2023-05-20 * "Coffee beans"
//!   Expenses:Groceries   10 CHF
//!   Assets:Checking
//! "#;
//!
//! // Parse into the `BeancountFile` struct:
//! let beancount: BeancountFile<f64> = input.parse()?;
//!
//! let directive = &beancount.directives[0];
//! assert_eq!(directive.date.year, 2023);
//! assert_eq!(directive.date.month, 5);
//! assert_eq!(directive.date.day, 20);
//!
//! let DirectiveContent::Transaction(trx) = &directive.content else {
//!     panic!("was not a transaction")
//! };
//! assert_eq!(trx.narration.as_deref(), Some("Coffee beans"));
//! assert_eq!(trx.postings[0].account.as_str(), "Expenses:Groceries");
//! assert_eq!(trx.postings[0].amount.as_ref().unwrap().value, 10.0);
//! assert_eq!(trx.postings[0].amount.as_ref().unwrap().currency.as_str(), "CHF");
//! assert_eq!(trx.postings[1].account.as_str(), "Assets:Checking");
//! assert_eq!(trx.postings[1].amount, None);
//! # Ok(()) }
//! ```

use std::{collections::HashSet, fs::File, io::Read, path::PathBuf, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{char, line_ending, not_line_ending, space0, space1},
    combinator::{all_consuming, cut, eof, iterator, map, not, opt},
    sequence::{delimited, preceded, terminated, tuple},
    Finish, Parser,
};
use nom_locate::position;

use crate::iterator::Iter;
pub use crate::{
    account::{Account, Balance, Close, Open, Pad},
    amount::{Amount, Currency, Decimal, Price},
    date::Date,
    error::{ConversionError, Error, ReadFileError},
    event::Event,
    transaction::{Cost, Link, Posting, PostingPrice, Tag, Transaction},
};

#[deprecated(note = "use `metadata::Value` instead", since = "1.0.0-beta.3")]
#[doc(hidden)]
pub type MetadataValue<D> = metadata::Value<D>;

mod account;
mod amount;
mod date;
mod error;
mod event;
mod iterator;
pub mod metadata;
mod transaction;

/// Parse the input beancount file and return an instance of [`BeancountFile`] on success
///
/// It is generic over the [`Decimal`] type `D`.
///
/// See the root crate documentation for an example.
///
/// # Errors
///
/// Returns an [`Error`] in case of invalid beancount syntax found.
pub fn parse<D: Decimal>(input: &str) -> Result<BeancountFile<D>, Error> {
    input.parse()
}

/// Parse the beancount file and return an iterator over `Result<Entry<D>, Result>`
///
/// It is generic over the [`Decimal`] type `D`.
///
/// See [`Entry`]
///
/// # Errors
///
/// The iterator will emit an [`Error`] in case of invalid beancount syntax found.
pub fn parse_iter<'a, D: Decimal + 'a>(
    input: &'a str,
) -> impl Iterator<Item = Result<Entry<D>, Error>> + 'a {
    Iter::new(iterator(Span::new(input), entry::<D>))
}

impl<D: Decimal> FromStr for BeancountFile<D> {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_iter(input).collect()
    }
}

/// Read the files from disk and parse their content.
///
/// It follows the `include` directives found.
///
/// # Errors
///
/// Returns an error if any file could not be read (IO error)
/// or if there is a beancount syntax error in any file read
pub fn read_files<D: Decimal, F: FnMut(Entry<D>)>(
    files: impl IntoIterator<Item = PathBuf>,
    mut on_entry: F,
) -> Result<(), ReadFileError> {
    let mut loaded: HashSet<PathBuf> = HashSet::new();
    let mut pending: Vec<PathBuf> = files
        .into_iter()
        .map(|p| p.canonicalize())
        .collect::<Result<_, _>>()?;
    let mut buffer = String::new();
    while let Some(path) = pending.pop() {
        if loaded.contains(&path) {
            continue;
        }
        loaded.insert(path.clone());
        buffer.clear();
        File::open(&path)?.read_to_string(&mut buffer)?;
        for result in parse_iter::<D>(&buffer) {
            let entry = result?;
            match entry {
                Entry::Include(include) => {
                    let path = if include.is_relative() {
                        let Some(parent) = path.parent() else { unreachable!("there must be a parent if the file was valid") };
                        parent.join(include)
                    } else {
                        include
                    };
                    let path = path.canonicalize()?;
                    if !loaded.contains(&path) {
                        pending.push(path);
                    }
                }
                entry => on_entry(entry),
            }
        }
    }
    Ok(())
}

/// Main struct representing a parsed beancount file.
///
/// To get an instance of this, use [`parse`].
///
/// For an example, look at the root crate documentation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BeancountFile<D> {
    /// List of beancount options
    ///
    /// See: <https://beancount.github.io/docs/beancount_language_syntax.html#options>
    pub options: Vec<BeanOption>,
    /// Paths of include directives
    ///
    /// See: <https://beancount.github.io/docs/beancount_language_syntax.html#includes>
    pub includes: Vec<PathBuf>,
    /// List of [`Directive`] found in the file
    pub directives: Vec<Directive<D>>,
}

impl<D> Default for BeancountFile<D> {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            includes: Vec::new(),
            directives: Vec::new(),
        }
    }
}

impl<D> BeancountFile<D> {
    /// Returns the first value found for the option
    ///
    /// If the option is declared multiple times, this function returns the first one found.
    ///
    /// See [`Self::options`] to get all declared options.
    ///
    /// Syntax: <https://beancount.github.io/docs/beancount_language_syntax.html#options>
    ///
    /// # Example
    ///
    /// ```
    /// use beancount_parser::BeancountFile;
    /// let input = r#"
    /// option "favorite_color" "blue"
    /// option "operating_currency" "CHF"
    /// option "operating_currency" "PLN"
    /// "#;
    /// let beancount: BeancountFile<f64> = input.parse().unwrap();
    /// assert_eq!(beancount.option("favorite_color"), Some("blue"));
    /// assert_eq!(beancount.option("operating_currency"), Some("CHF"));
    /// assert_eq!(beancount.option("foo"), None);
    /// ```
    #[must_use]
    pub fn option(&self, key: &str) -> Option<&str> {
        self.options
            .iter()
            .find(|opt| opt.name == key)
            .map(|opt| &opt.value[..])
    }
}

impl<D> Extend<Entry<D>> for BeancountFile<D> {
    fn extend<T: IntoIterator<Item = Entry<D>>>(&mut self, iter: T) {
        for entry in iter {
            match entry {
                Entry::Directive(d) => self.directives.push(d),
                Entry::Option(o) => self.options.push(o),
                Entry::Include(p) => self.includes.push(p),
            }
        }
    }
}

impl<D> FromIterator<Entry<D>> for BeancountFile<D> {
    fn from_iter<T: IntoIterator<Item = Entry<D>>>(iter: T) -> Self {
        let mut file = BeancountFile::default();
        file.extend(iter);
        file
    }
}

/// A beancount "directive"
///
/// It has fields common to all directives, and a [`Directive::content`] field with
/// a different content for each directive type.
///
/// ```
/// # use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = r#"
/// 2022-01-01 open Assets:Cash
/// 2022-01-01 * "Grocery shopping"
///   Expenses:Groceries  10 CHF
///   Assets:Cash
/// "#;
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
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
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Directive<D> {
    /// Date of the directive
    pub date: Date,
    /// Content of the directive that is specific to each directive type
    pub content: DirectiveContent<D>,
    /// Metadata associated to the directive
    ///
    /// See the [`metadata`] module for more
    pub metadata: metadata::Map<D>,
    /// Line number where the directive was found in the input file
    pub line_number: u32,
}

impl<D: Decimal> FromStr for Directive<D> {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(directive)(Span::new(s)).finish() {
            Ok((_, d)) => Ok(d),
            Err(err) => Err(Error::new(err.input)),
        }
    }
}

/// Directive specific content
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DirectiveContent<D> {
    Transaction(Transaction<D>),
    Price(Price<D>),
    Balance(Balance<D>),
    Open(Open),
    Close(Close),
    Pad(Pad),
    Commodity(Currency),
    Event(Event),
}

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;

/// Entry in the beancount syntax
///
/// It is more general than `Directive` as an entry can also be option or an include.
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Entry<D> {
    Directive(Directive<D>),
    Option(BeanOption),
    Include(PathBuf),
}

enum RawEntry<D> {
    Directive(Directive<D>),
    Option(BeanOption),
    Include(PathBuf),
    PushTag(Tag),
    PopTag(Tag),
    Comment,
}

/// An beancount option
///
/// See: <https://beancount.github.io/docs/beancount_language_syntax.html#options>
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BeanOption {
    /// Name of the option
    pub name: String,
    /// Value of the option
    pub value: String,
}

fn entry<D: Decimal>(input: Span<'_>) -> IResult<'_, RawEntry<D>> {
    alt((
        directive.map(RawEntry::Directive),
        option.map(|(name, value)| {
            RawEntry::Option(BeanOption {
                name: name.into(),
                value: value.into(),
            })
        }),
        include.map(|p| RawEntry::Include(p)),
        tag_stack_operation,
        line.map(|_| RawEntry::Comment),
    ))(input)
}

fn directive<D: Decimal>(input: Span<'_>) -> IResult<'_, Directive<D>> {
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

fn include(input: Span<'_>) -> IResult<'_, PathBuf> {
    let (input, _) = tag("include")(input)?;
    let (input, path) = cut(delimited(space1, string, end_of_line))(input)?;
    Ok((input, path.into()))
}

fn tag_stack_operation<D>(input: Span<'_>) -> IResult<'_, RawEntry<D>> {
    alt((
        preceded(tuple((tag("pushtag"), space1)), transaction::parse_tag).map(RawEntry::PushTag),
        preceded(tuple((tag("poptag"), space1)), transaction::parse_tag).map(RawEntry::PopTag),
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

fn empty_line(input: Span<'_>) -> IResult<'_, ()> {
    let (input, _) = not(eof)(input)?;
    end_of_line(input)
}

fn string(input: Span<'_>) -> IResult<'_, &str> {
    map(
        delimited(char('"'), take_till(|c: char| c == '"'), char('"')),
        |s: Span<'_>| *s.fragment(),
    )(input)
}
