//! Types for representing an [`Amount`]

use nom::{
    bytes::complete::take_while1, character::complete::space1, combinator::map,
    sequence::separated_pair, IResult,
};

pub use self::expression::{ConversionError, Expression, Value};

mod expression;
mod sum;

/// A beancount amount
///
/// The amount is the combination of an [`Expression`] and the currency.
#[derive(Debug, Clone, PartialEq)]
pub struct Amount<'a> {
    expression: Expression,
    currency: &'a str,
}

impl<'a> Amount<'a> {
    #[cfg(any(test, feature = "unstable"))]
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

pub(crate) fn amount(input: &str) -> IResult<&str, Amount<'_>> {
    map(
        separated_pair(expression::parse, space1, currency),
        |(expression, currency)| Amount {
            expression,
            currency,
        },
    )(input)
}

pub(crate) fn currency(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_uppercase() && c.is_ascii_alphabetic())(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_amount() {
        assert_eq!(
            amount("10 CHF"),
            Ok((
                "",
                Amount {
                    expression: Expression::value(10),
                    currency: "CHF"
                }
            ))
        );
    }

    #[test]
    fn invalid_amount() {
        assert!(amount("10 chf").is_err());
    }
}
