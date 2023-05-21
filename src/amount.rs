#![allow(missing_docs)]

use std::str::FromStr;

use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete::{satisfy, space1},
    combinator::{map_res, recognize, verify},
    sequence::tuple,
};

use crate::{IResult, Span};

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Amount<'a, D> {
    pub value: D,
    pub currency: Currency<'a>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Currency<'a>(&'a str);

impl<'a> Currency<'a> {
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug)]
pub struct Price<'a, D> {
    pub currency: Currency<'a>,
    pub amount: Amount<'a, D>,
}

pub(crate) fn parse<D: FromStr>(input: Span<'_>) -> IResult<'_, Amount<'_, D>> {
    let (input, value) = value(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Amount { value, currency }))
}

fn value<D: FromStr>(input: Span<'_>) -> IResult<'_, D> {
    map_res(
        take_while1(|c: char| c.is_numeric() || c == '-' || c == '.'),
        |s: Span<'_>| s.fragment().parse(),
    )(input)
}

pub(crate) fn price<D: FromStr>(input: Span<'_>) -> IResult<'_, Price<'_, D>> {
    let (input, currency) = currency(input)?;
    let (input, _) = space1(input)?;
    let (input, amount) = parse(input)?;
    Ok((input, Price { currency, amount }))
}

pub(crate) fn currency(input: Span<'_>) -> IResult<'_, Currency<'_>> {
    let (input, currency) = recognize(tuple((
        satisfy(char::is_uppercase),
        verify(
            take_while(|c: char| {
                c.is_uppercase() || c.is_numeric() || c == '-' || c == '_' || c == '.' || c == '\''
            }),
            |s: &Span<'_>| s.fragment().chars().last().map_or(true, char::is_uppercase),
        ),
    )))(input)?;
    Ok((input, Currency(currency.fragment())))
}
