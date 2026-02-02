use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter},
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
    sync::Arc,
};

use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, one_of, satisfy, space0, space1},
    combinator::{all_consuming, iterator, map_res, opt, recognize, verify},
    sequence::{delimited, preceded, terminated},
    Finish, Parser,
};

use crate::{IResult, Span};

/// Price directive
///
/// # Example
///
/// ```
/// use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = "2023-05-27 price CHF  4 PLN";
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Price(price) = &beancount.directives[0].content else { unreachable!() };
/// assert_eq!(price.currency.as_str(), "CHF");
/// assert_eq!(price.amount.value, 4.0);
/// assert_eq!(price.amount.currency.as_str(), "PLN");
/// ```
#[derive(Debug, Clone, PartialEq)]
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
        value.parse().map_err(|_| crate::ConversionError)
    }
}

impl FromStr for Currency {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let span = Span::new(s);
        match all_consuming(currency).parse(span).finish() {
            Ok((_, currency)) => Ok(currency),
            Err(_) => Err(crate::Error::new(s, span)),
        }
    }
}

pub(crate) fn parse<D: Decimal>(input: Span<'_>) -> IResult<'_, Amount<D>> {
    let (input, value) = expression(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Amount { value, currency }))
}

pub(crate) fn expression<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    alt((negation, sum)).parse(input)
}

fn sum<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, value) = product(input)?;
    let mut iter = iterator(input, (delimited(space0, one_of("+-"), space0), product));
    let value = iter.by_ref().fold(value, |a, (op, b)| match op {
        '+' => a + b,
        '-' => a - b,
        op => unreachable!("unsupported operator: {}", op),
    });
    let (input, ()) = iter.finish()?;
    Ok((input, value))
}

fn product<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, value) = atom(input)?;
    let mut iter = iterator(input, (delimited(space0, one_of("*/"), space0), atom));
    let value = iter.by_ref().fold(value, |a, (op, b)| match op {
        '*' => a * b,
        '/' => a / b,
        op => unreachable!("unsupported operator: {}", op),
    });
    let (input, ()) = iter.finish()?;
    Ok((input, value))
}

fn atom<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    alt((literal, group)).parse(input)
}

fn group<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    delimited(
        terminated(char('('), space0),
        expression,
        preceded(space0, char(')')),
    )
    .parse(input)
}

fn negation<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, _) = char('-')(input)?;
    let (input, _) = space0(input)?;
    let (input, expr) = group::<D>(input)?;
    Ok((input, -expr))
}

fn literal<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    map_res(
        recognize((
            opt(char('-')),
            space0,
            take_while1(|c: char| c.is_numeric() || c == '.' || c == ','),
        )),
        |s: Span<'_>| s.fragment().replace([',', ' '], "").parse(),
    )
    .parse(input)
}

pub(crate) fn price<D: Decimal>(input: Span<'_>) -> IResult<'_, Price<D>> {
    let (input, currency) = currency(input)?;
    let (input, _) = space1(input)?;
    let (input, amount) = parse(input)?;
    Ok((input, Price { currency, amount }))
}

pub(crate) fn currency(input: Span<'_>) -> IResult<'_, Currency> {
    let (input, currency) = recognize((
        satisfy(char::is_uppercase),
        verify(
            take_while(|c: char| {
                c.is_uppercase() || c.is_numeric() || c == '-' || c == '_' || c == '.' || c == '\''
            }),
            |s: &Span<'_>| {
                s.fragment()
                    .chars()
                    .last()
                    .map_or(true, |c| c.is_uppercase() || c.is_numeric())
            },
        ),
    ))
    .parse(input)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("CHF")]
    fn currency_from_str_should_parse_valid_currency(#[case] input: &str) {
        let currency: Currency = input.parse().unwrap();
        assert_eq!(currency.as_str(), input);
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    #[case("oops")]
    fn currency_from_str_should_not_parse_invalid_currency(#[case] input: &str) {
        let currency: Result<Currency, _> = input.parse();
        assert!(currency.is_err(), "{currency:?}");
    }
}
