//! Types for representing an [`Amount`]

use nom::{
    branch::alt,
    character::complete::{one_of, satisfy, space1},
    combinator::{map, not, opt, peek, recognize},
    multi::many_till,
    sequence::{pair, separated_pair},
};

use crate::{IResult, Span};

pub use self::expression::{ConversionError, Expression, Value};

pub(crate) mod expression;

/// A beancount amount
///
/// The amount is the combination of an [`Expression`] and the currency.
#[derive(Debug, Clone, PartialEq)]
pub struct Amount<'a> {
    pub(crate) expression: Expression,
    pub(crate) currency: &'a str,
}

impl<'a> Amount<'a> {
    #[cfg(any(test))]
    pub(crate) fn new(value: impl Into<rust_decimal::Decimal>, currency: &'a str) -> Self {
        Self {
            expression: Expression::value(value),
            currency,
        }
    }

    /// Returns the [`Expression`] which may be inspected or evaluated
    #[must_use]
    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    /// Evaluate the expression and returns the value
    #[must_use]
    pub fn value(&self) -> Value {
        self.expression.evaluate()
    }

    /// Returns the currency
    #[must_use]
    pub fn currency(&self) -> &'a str {
        self.currency
    }
}

pub(crate) fn amount(input: Span<'_>) -> IResult<'_, Amount<'_>> {
    map(
        separated_pair(expression::parse, space1, currency),
        |(expression, currency)| Amount {
            expression,
            currency,
        },
    )(input)
}

fn current_first_char(input: Span<'_>) -> IResult<'_, char> {
    satisfy(|c: char| c.is_ascii_uppercase() && c.is_ascii_alphabetic())(input)
}

fn current_middle_char(input: Span<'_>) -> IResult<'_, char> {
    alt((
        satisfy(|c: char| c.is_ascii_uppercase() && c.is_ascii_alphabetic()),
        satisfy(char::is_numeric),
        one_of("'._-"),
    ))(input)
}

fn current_last_char(input: Span<'_>) -> IResult<'_, char> {
    alt((
        satisfy(|c: char| c.is_ascii_uppercase() && c.is_ascii_alphabetic()),
        satisfy(char::is_numeric),
    ))(input)
}

pub(crate) fn currency(input: Span<'_>) -> IResult<'_, &str> {
    map(
        recognize(pair(
            current_first_char,
            opt(pair(
                many_till(
                    current_middle_char,
                    peek(pair(current_last_char, not(current_middle_char))),
                ),
                current_last_char,
            )),
        )),
        |s: Span<'_>| *s.fragment(),
    )(input)
}

#[cfg(test)]
mod tests {
    use nom::combinator::all_consuming;

    use super::*;

    #[test]
    fn parse_amount() {
        assert_eq!(
            amount(Span::new("10 CHF")).unwrap().1,
            Amount {
                expression: Expression::value(10),
                currency: "CHF"
            }
        );
    }

    #[test]
    fn invalid_amount() {
        assert!(amount(Span::new("10 chf")).is_err());
    }

    #[rstest]
    fn valid_currency(#[values("CHF", "X-A", "X_A", "X'A", "A", "AB", "A2", "R2D2")] input: &str) {
        assert_eq!(all_consuming(currency)(Span::new(input)).unwrap().1, input);
    }

    #[rstest]
    fn invalid_currency(#[values("CHF-", "X-a", "1A", "aA")] input: &str) {
        let p = all_consuming(currency)(Span::new(input));
        assert!(p.is_err(), "Result was actually: {p:#?}");
    }
}
