use nom::{
    bytes::complete::take_while,
    character::complete::satisfy,
    combinator::{recognize, verify},
    sequence::tuple,
};

use crate::{IResult, Span};

#[derive(Debug)]
pub struct Currency<'a>(&'a str);

impl<'a> Currency<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

pub(crate) fn parse(input: Span<'_>) -> IResult<'_, Currency<'_>> {
    let (input, currency) = recognize(tuple((
        satisfy(|c: char| c.is_uppercase()),
        verify(
            take_while(|c: char| {
                c.is_uppercase() || c.is_numeric() || c == '-' || c == '_' || c == '.' || c == '\''
            }),
            |s: &Span<'_>| {
                s.fragment()
                    .chars()
                    .last()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(true)
            },
        ),
    )))(input)?;
    Ok((input, Currency(currency.fragment())))
}
