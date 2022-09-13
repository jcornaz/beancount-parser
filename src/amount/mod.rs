use nom::{
    bytes::complete::take_while1, character::complete::space1, combinator::map,
    sequence::separated_pair, IResult,
};

pub use self::expression::{Expression, Value};

mod expression;

#[derive(Debug, Clone, PartialEq)]
pub struct Amount<'a> {
    expression: Expression,
    currency: &'a str,
}

impl<'a> Amount<'a> {
    #[cfg(test)]
    pub(crate) fn new(value: impl Into<rust_decimal::Decimal>, currency: &'a str) -> Self {
        Self {
            expression: Expression::value(value),
            currency,
        }
    }

    #[must_use]
    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    #[must_use]
    pub fn value(&self) -> Value {
        self.expression.evaluate()
    }

    #[must_use]
    pub fn currency(&self) -> &str {
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

fn currency(input: &str) -> IResult<&str, &str> {
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
