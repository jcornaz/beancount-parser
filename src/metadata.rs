#![allow(missing_docs)]

use std::collections::HashMap;

use nom::{
    bytes::complete::take_while,
    character::complete::{char, satisfy, space1},
    combinator::{iterator, map, recognize},
    sequence::preceded,
};

use crate::{end_of_line, string, IResult, Span};

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Value<'a> {
    String(&'a str),
}

pub(crate) fn parse(input: Span<'_>) -> IResult<'_, HashMap<&str, Value<'_>>> {
    let mut iter = iterator(input, entry);
    let map: HashMap<_, _> = iter.collect();
    let (input, _) = iter.finish()?;
    Ok((input, map))
}

fn entry(input: Span<'_>) -> IResult<'_, (&str, Value<'_>)> {
    let (input, _) = space1(input)?;
    let (input, key) = recognize(preceded(
        satisfy(char::is_lowercase),
        take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
    ))(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = map(string, Value::String)(input)?;
    let (input, _) = end_of_line(input)?;
    Ok((input, (*key.fragment(), value)))
}
