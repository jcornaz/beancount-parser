#![allow(missing_docs)]

use std::{
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete::{satisfy, space1},
    combinator::{map_res, recognize, verify},
    sequence::tuple,
};

use crate::{IResult, Span};

/// Decimal type to which amount values and expressions will be parsed into.
///
/// # Notable implementations
///
/// * `f64`
/// * `Decimal` of the crate [rust_decimal]
///
/// [rust_decimal]: https://docs.rs/rust_decimal
///
pub trait Decimal:
    FromStr
    + Default
    + Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Neg<Output = Self>
    + PartialEq
    + PartialOrd
{
}

impl<D> Decimal for D where
    D: FromStr
        + Default
        + Copy
        + Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + SubAssign
        + Mul<Output = Self>
        + MulAssign
        + Div<Output = Self>
        + DivAssign
        + Neg<Output = Self>
        + PartialEq
        + PartialOrd
{
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Amount<'a, D> {
    pub value: D,
    pub currency: Currency<'a>,
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency<'a>(&'a str);

impl<'a> Currency<'a> {
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Price<'a, D> {
    pub currency: Currency<'a>,
    pub amount: Amount<'a, D>,
}

pub(crate) fn parse<D: Decimal>(input: Span<'_>) -> IResult<'_, Amount<'_, D>> {
    let (input, value) = value(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Amount { value, currency }))
}

fn value<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    map_res(
        take_while1(|c: char| c.is_numeric() || c == '-' || c == '.'),
        |s: Span<'_>| s.fragment().parse(),
    )(input)
}

pub(crate) fn price<D: Decimal>(input: Span<'_>) -> IResult<'_, Price<'_, D>> {
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
