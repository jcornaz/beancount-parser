use std::{collections::HashMap, hash::Hash};

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{char, satisfy, space1},
    combinator::{iterator, map, recognize},
    sequence::preceded,
};

use crate::{amount, end_of_line, string, Currency, Decimal, IResult, Span};

/// Metadata value
///
/// # Example
///
/// ```
/// # use beancount_parser_2::MetadataValue;
/// let input = r#"
/// 2023-05-27 commodity CHF
///     title: "Swiss Franc"
/// "#;
/// let beancount = beancount_parser_2::parse::<&str, f64>(input).unwrap();
/// let directive_metadata = &beancount.directives[0].metadata;
/// assert_eq!(directive_metadata.get("title"), Some(&MetadataValue::String("Swiss Franc")));
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Value<S, D> {
    /// String value
    String(S),
    /// A number or number expression
    Number(D),
    /// A [`Currency`]
    Currency(Currency<S>),
}

pub(crate) fn parse<'a, S: From<&'a str> + Eq + Hash, D: Decimal>(
    input: Span<'a>,
) -> IResult<'a, HashMap<S, Value<S, D>>> {
    let mut iter = iterator(input, entry);
    let map: HashMap<_, _> = iter.collect();
    let (input, _) = iter.finish()?;
    Ok((input, map))
}

fn entry<'a, S: From<&'a str>, D: Decimal>(input: Span<'a>) -> IResult<'a, (S, Value<S, D>)> {
    let (input, _) = space1(input)?;
    let (input, key) = recognize(preceded(
        satisfy(char::is_lowercase),
        take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
    ))(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = alt((
        map(string, Value::String),
        map(amount::expression, Value::Number),
        map(amount::currency, Value::Currency),
    ))(input)?;
    let (input, _) = end_of_line(input)?;
    Ok((input, ((*key.fragment()).into(), value)))
}
