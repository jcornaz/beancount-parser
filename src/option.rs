#![cfg(feature = "unstable")]

use nom::{
    bytes::complete::tag, bytes::streaming::take_till, character::complete::char,
    character::complete::space1, sequence::delimited, sequence::terminated,
};

use crate::{IResult, Span};

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
