use std::borrow::Borrow;
use std::{
    fmt::Debug,
    fmt::{Display, Formatter},
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
    sync::Arc,
};

use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, one_of, satisfy, space0, space1},
    combinator::all_consuming,
    combinator::{iterator, map_res, opt, recognize, verify},
    sequence::{delimited, preceded, terminated, tuple},
    Finish,
};

use crate::{IResult, Span};

/// Price directive
///
/// # Example
///
/// ```
/// use beancount_parser_2::DirectiveContent;
/// let input = "2023-05-27 price CHF  4 PLN";
/// let beancount = beancount_parser_2::parse::<f64>(input).unwrap();
/// let DirectiveContent::Price(price) = &beancount.directives[0].content else { unreachable!() };
/// assert_eq!(price.currency.as_str(), "CHF");
/// assert_eq!(price.amount.value, 4.0);
/// assert_eq!(price.amount.currency.as_str(), "PLN");
/// ```
#[derive(Debug, Clone)]
pub struct Price<D> {
    /// Currency
    pub currency: Currency,
    /// Price of the currency
    pub amount: Amount<D>,
}

/// Amount
///
/// Where `D` is the decimal type (like `f64` or `rust_decimal::Decimal`)
///
/// For an example, look at the [`Price`] directive
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Amount<D> {
    /// The value (decimal) part
    pub value: D,
    /// Currency
    pub currency: Currency,
}

/// Currency
///
/// One may use [`Currency::as_str`] to get the string representation of the currency
///
/// For an example, look at the [`Price`] directive
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency(Arc<str>);

impl Currency {
    /// Returns underlying string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Currency {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for Currency {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl<'a> TryFrom<&'a str> for Currency {
    type Error = crate::ConversionError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match all_consuming(currency)(Span::new(value)).finish() {
            Ok((_, currency)) => Ok(currency),
            Err(_) => Err(crate::ConversionError),
        }
    }
}

pub(crate) fn parse<D: Decimal>(input: Span<'_>) -> IResult<'_, Amount<D>> {
    let (input, value) = expression(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Amount { value, currency }))
}

pub(super) fn expression<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    exp_p2(input)
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
        op => unreachable!("unsupported operator: {}", op),
    });
    let (input, _) = iter.finish()?;
    Ok((input, value))
}

fn exp_p1<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, value) = exp_p0(input)?;
    let mut iter = iterator(
        input,
        tuple((delimited(space0, one_of("*/"), space0), exp_p0)),
    );
    let value = iter.fold(value, |a, (op, b)| match op {
        '*' => a * b,
        '/' => a / b,
        op => unreachable!("unsupported operator: {}", op),
    });
    let (input, _) = iter.finish()?;
    Ok((input, value))
}

fn exp_p0<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    alt((
        literal,
        delimited(
            terminated(char('('), space0),
            expression,
            preceded(space0, char(')')),
        ),
    ))(input)
}

fn literal<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            take_while1(|c: char| c.is_numeric() || c == '.'),
        ))),
        |s: Span<'_>| s.fragment().parse(),
    )(input)
}

pub(crate) fn price<D: Decimal>(input: Span<'_>) -> IResult<'_, Price<D>> {
    let (input, currency) = currency(input)?;
    let (input, _) = space1(input)?;
    let (input, amount) = parse(input)?;
    Ok((input, Price { currency, amount }))
}

pub(crate) fn currency(input: Span<'_>) -> IResult<'_, Currency> {
    let (input, currency) = recognize(tuple((
        satisfy(char::is_uppercase),
        verify(
            take_while(|c: char| {
                c.is_uppercase() || c.is_numeric() || c == '-' || c == '_' || c == '.' || c == '\''
            }),
            |s: &Span<'_>| s.fragment().chars().last().map_or(true, char::is_uppercase),
        ),
    )))(input)?;
    Ok((input, Currency(Arc::from(*currency.fragment()))))
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
    + Clone
    + Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + PartialEq
    + PartialOrd
{
}

impl<D> Decimal for D where
    D: FromStr
        + Default
        + Clone
        + Debug
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + PartialEq
        + PartialOrd
{
}
