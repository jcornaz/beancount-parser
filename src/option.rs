#![cfg(feature = "unstable")]

use crate::pest_parser::Pair;
use crate::{string, IResult, Span};
use nom::bytes::streaming::take_till;
use nom::character::complete::space1;
use nom::sequence::delimited;
use nom::{bytes::complete::tag, character::complete::char, sequence::terminated};

/// beancount option
///
/// see: <https://beancount.github.io/docs/beancount_language_syntax.html#options>
#[derive(Debug, Clone)]
pub struct Option<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> Option<'a> {
    #[must_use]
    pub fn name(&self) -> &'a str {
        self.name
    }

    #[must_use]
    pub fn value(&self) -> &'a str {
        self.value
    }

    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let name = string::from_pair(inner.next().expect("no name in option"));
        let value = string::from_pair(inner.next().expect("no name in option"));
        Self { name, value }
    }
}

pub(crate) fn option(input: Span<'_>) -> IResult<'_, Option<'_>> {
    let (input, _) = terminated(tag("option"), space1)(input)?;
    let (input, name) = delimited(char('"'), take_till(|c| c == '"'), char('"'))(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = delimited(char('"'), take_till(|c| c == '"'), char('"'))(input)?;
    Ok((
        input,
        Option {
            name: name.fragment(),
            value: value.fragment(),
        },
    ))
}
