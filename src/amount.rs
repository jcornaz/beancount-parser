use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete::{satisfy, space1},
    combinator::{map_res, recognize, verify},
    sequence::tuple,
};

use crate::{Decimal, IResult, Span};

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Amount<'a> {
    pub value: Decimal,
    pub currency: Currency<'a>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Currency<'a>(&'a str);

impl<'a> Currency<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug)]
pub struct Price<'a> {
    pub currency: Currency<'a>,
    pub amount: Amount<'a>,
}

pub(crate) fn parse(input: Span<'_>) -> IResult<'_, Amount<'_>> {
    let (input, value) = value(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Amount { value, currency }))
}

fn value(input: Span<'_>) -> IResult<'_, Decimal> {
    map_res(
        take_while1(|c: char| c.is_numeric() || c == '-' || c == '.'),
        |s: Span<'_>| s.fragment().parse(),
    )(input)
}

pub(crate) fn price(input: Span<'_>) -> IResult<'_, Price<'_>> {
    let (input, currency) = currency(input)?;
    let (input, _) = space1(input)?;
    let (input, amount) = parse(input)?;
    Ok((input, Price { currency, amount }))
}

pub(crate) fn currency(input: Span<'_>) -> IResult<'_, Currency<'_>> {
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
