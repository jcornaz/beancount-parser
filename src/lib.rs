#![deny(future_incompatible, nonstandard_style, unsafe_code, private_in_public)]
#![warn(rust_2018_idioms, clippy::pedantic)]

mod account;
mod amount;
mod date;
mod event;
pub mod metadata;
mod transaction;

use amount::Currency;
pub use date::Date;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{char, line_ending, not_line_ending, space0, space1},
    combinator::{all_consuming, cut, eof, iterator, map, opt},
    sequence::{delimited, preceded, terminated, tuple},
    Finish, Parser,
};
pub use rust_decimal::Decimal;
use std::collections::HashMap;
pub use transaction::{Flag, Posting, Transaction};

/// Parse the input beancount file and return an instance of [`BeancountFile`] on success
///
/// # Errors
///
/// Returns an error in case of invalid beancount syntax found
pub fn parse(input: &str) -> Result<BeancountFile<'_>, Error<'_>> {
    match all_consuming(beancount_file)(Span::new(input)).finish() {
        Ok((_, content)) => Ok(content),
        Err(nom::error::Error { input, .. }) => Err(Error(input)),
    }
}

#[derive(Debug)]
pub struct Error<'a>(Span<'a>);

#[derive(Debug)]
#[non_exhaustive]
pub struct BeancountFile<'a> {
    pub options: HashMap<&'a str, &'a str>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Directive<'a> {
    pub date: Date,
    pub content: DirectiveContent<'a>,
    pub metadata: HashMap<&'a str, metadata::Value<'a>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DirectiveContent<'a> {
    Transaction(transaction::Transaction<'a>),
    Price(amount::Price<'a>),
    Balance(account::Balance<'a>),
    Open(account::Open<'a>),
    Close(account::Close<'a>),
    Commodity(Currency<'a>),
    Event(event::Event<'a>),
}

type Span<'a> = nom_locate::LocatedSpan<&'a str>;
type IResult<'a, O> = nom::IResult<Span<'a>, O>;

fn beancount_file(input: Span<'_>) -> IResult<'_, BeancountFile<'_>> {
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

enum Entry<'a> {
    Directive(Directive<'a>),
    Option { key: &'a str, value: &'a str },
    Comment,
}

fn entry(input: Span<'_>) -> IResult<'_, Entry<'_>> {
    alt((
        directive.map(Entry::Directive),
        line.map(|_| Entry::Comment),
        option.map(|(key, value)| Entry::Option { key, value }),
    ))(input)
}

fn directive(input: Span<'_>) -> IResult<'_, Directive<'_>> {
    let (input, date) = date::parse(input)?;
    let (input, (content, metadata)) = cut(preceded(
        space1,
        alt((
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
        )),
    ))(input)?;
    Ok((
        input,
        Directive {
            date,
            content,
            metadata,
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
