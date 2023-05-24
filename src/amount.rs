#![allow(missing_docs)]

use std::{
    fmt::Debug,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete::{char, one_of, satisfy, space0, space1},
    combinator::{iterator, map_res, opt, recognize, verify},
    sequence::{delimited, tuple},
};

use crate::{IResult, Span};

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
    let (input, value) = exp_p2(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Amount { value, currency }))
}

fn exp_p2<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, value) = exp_p1(input)?;
    let mut iter = iterator(
        input,
        tuple((delimited(space0, one_of("+-"), space0), exp_p1)),
    );
    let value = iter.fold(value, |a, (op, b)| match op {
        '+' => a + b,
        '-' => a - b,
        op => unreachable!("unsupported operator: {op}"),
    });
    let (input, _) = iter.finish()?;
    Ok((input, value))
}

fn exp_p1<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, value) = exp_p0(input)?;
    let mut iter = iterator(
        input,
        tuple((delimited(space0, one_of("*"), space0), exp_p0)),
    );
    let value = iter.fold(value, |a, (op, b)| match op {
        '*' => a * b,
        op => unreachable!("unsupported operator: {op}"),
    });
    let (input, _) = iter.finish()?;
    Ok((input, value))
}

fn exp_p0<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            take_while1(|c: char| c.is_numeric() || c == '.'),
        ))),
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
    + Debug
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Sum
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Product
    + Neg<Output = Self>
    + PartialEq
    + PartialOrd
{
}

impl<D> Decimal for D where
    D: FromStr
        + Default
        + Copy
        + Debug
        + Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + SubAssign
        + Sum
        + Mul<Output = Self>
        + MulAssign
        + Div<Output = Self>
        + DivAssign
        + Product
        + Neg<Output = Self>
        + PartialEq
        + PartialOrd
{
}
