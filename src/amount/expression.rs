use nom::{
    branch::alt,
    character::complete::{char, digit0, digit1, space0},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use thiserror::Error;

/// An expression
///
/// Expple of expressions:
///
/// * `42`
/// * `2 + 2`
/// * `5 / (6 + (8 / 2))`
///
/// The expression can be evaluated with [`Expression::evaluate`]
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Expression {
    /// A direct value, the leaf of the expression tree
    Value(Value),

    /// An operation, made of two expression operands
    Operation(Operation),
}

/// A direct value, the leaf of the expression tree
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct Value(Decimal);

/// An operation, made of two expression operands
#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    operator: Operator,
    left: Box<Expression>,
    right: Box<Expression>,
}

impl Operation {
    /// Returns the operator
    #[must_use]
    pub fn operator(&self) -> Operator {
        self.operator
    }

    /// Returns the left operand
    #[must_use]
    pub fn left(&self) -> &Expression {
        &self.left
    }

    /// Returns the right operand
    #[must_use]
    pub fn right(&self) -> &Expression {
        &self.right
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum Operator {
    Divide,
    Multiply,
    Add,
    Substract,
}

impl Expression {
    #[cfg(test)]
    pub(super) fn value(dec: impl Into<Decimal>) -> Self {
        Self::Value(Value(dec.into()))
    }

    fn from_iter(left: Self, right: impl IntoIterator<Item = (Operator, Self)>) -> Self {
        right.into_iter().fold(left, |left, (operator, right)| {
            Expression::operation(operator, left, right)
        })
    }

    fn operation(operator: Operator, left: Self, right: Self) -> Self {
        Self::Operation(Operation {
            operator,
            left: left.into(),
            right: right.into(),
        })
    }

    #[cfg(test)]
    fn div(left: Self, right: Self) -> Self {
        Self::operation(Operator::Divide, left, right)
    }

    #[cfg(test)]
    fn mul(left: Self, right: Self) -> Self {
        Self::operation(Operator::Multiply, left, right)
    }

    #[cfg(test)]
    fn plus(left: Self, right: Self) -> Self {
        Self::operation(Operator::Add, left, right)
    }

    #[cfg(test)]
    fn minus(left: Self, right: Self) -> Self {
        Self::operation(Operator::Substract, left, right)
    }

    /// Evaluate the expression
    #[must_use]
    pub fn evaluate(&self) -> Value {
        match self {
            Expression::Value(value) => *value,
            Expression::Operation(Operation {
                operator,
                left,
                right,
            }) => operator.evaluate(left.evaluate(), right.evaluate()),
        }
    }
}

impl Operator {
    fn evaluate(self, Value(left): Value, Value(right): Value) -> Value {
        Value(match self {
            Operator::Divide => left / right,
            Operator::Multiply => left * right,
            Operator::Add => left + right,
            Operator::Substract => left - right,
        })
    }
}

impl Value {
    /// Try to convert this value into a `f64`
    ///
    /// # Errors
    ///
    /// Returns an error in case of overfow
    pub fn try_into_f64(self) -> Result<f64, ConversionError> {
        self.try_into()
    }

    /// Try to convert this value into a `f32`
    ///
    /// # Errors
    ///
    /// Returns an error in case of overfow
    pub fn try_into_f32(self) -> Result<f32, ConversionError> {
        self.try_into()
    }
}

impl TryFrom<Value> for f64 {
    type Error = ConversionError;

    fn try_from(Value(v): Value) -> Result<Self, Self::Error> {
        v.to_f64().ok_or(ConversionError(v))
    }
}

impl TryFrom<Value> for f32 {
    type Error = ConversionError;

    fn try_from(Value(v): Value) -> Result<Self, Self::Error> {
        v.to_f32().ok_or(ConversionError(v))
    }
}

#[cfg(feature = "rust_decimal")]
impl From<Value> for rust_decimal::Decimal {
    fn from(Value(v): Value) -> Self {
        v
    }
}

/// Error returned when a [`Value`] cannot be converted to the desired type
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
#[error("Cannot convert {0} into the desired type")]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct ConversionError(Decimal);

impl From<ConversionError> for crate::Error {
    fn from(_: ConversionError) -> Self {
        Self
    }
}

pub(super) fn parse(input: &str) -> IResult<&str, Expression> {
    exp_p2(input)
}

fn exp_p0(input: &str) -> IResult<&str, Expression> {
    alt((
        delimited(
            tuple((char('('), space0)),
            exp_p2,
            tuple((space0, char(')'))),
        ),
        map(value, Expression::Value),
    ))(input)
}

fn exp_p1(input: &str) -> IResult<&str, Expression> {
    let operator = alt((
        map(char('*'), |_| Operator::Multiply),
        map(char('/'), |_| Operator::Divide),
    ));
    map(
        tuple((
            exp_p0,
            many0(tuple((delimited(space0, operator, space0), exp_p0))),
        )),
        |(left, right)| Expression::from_iter(left, right),
    )(input)
}

fn exp_p2(input: &str) -> IResult<&str, Expression> {
    let operator = alt((
        map(char('+'), |_| Operator::Add),
        map(char('-'), |_| Operator::Substract),
    ));
    map(
        tuple((
            exp_p1,
            many0(tuple((delimited(space0, operator, space0), exp_p1))),
        )),
        |(left, right)| Expression::from_iter(left, right),
    )(input)
}

fn value(input: &str) -> IResult<&str, Value> {
    let value_string = recognize(tuple((
        opt(char('-')),
        digit0,
        opt(preceded(char('.'), digit1)),
    )));
    map(map_res(value_string, str::parse), Value)(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("0", Decimal::ZERO)]
    #[case("42", Decimal::new(42, 0))]
    #[case("1.1", Decimal::new(11, 1))]
    #[case(".1", Decimal::new(1, 1))]
    #[case("-2", -Decimal::new(2, 0))]
    fn parse_value(#[case] input: &str, #[case] expected: Decimal) {
        assert_eq!(parse(input), Ok(("", Expression::Value(Value(expected)))));
    }

    #[rstest]
    #[case("3 / 2", Expression::div(Expression::value(3), Expression::value(2)))]
    #[case("3 * 2", Expression::mul(Expression::value(3), Expression::value(2)))]
    #[case("3 + 2", Expression::plus(Expression::value(3), Expression::value(2)))]
    #[case("3 - 2", Expression::minus(Expression::value(3), Expression::value(2)))]
    #[case(
        "3 - 2 - 1",
        Expression::minus(
            Expression::minus(Expression::value(3), Expression::value(2)),
            Expression::value(1)
        )
    )]
    #[case(
        "3 * 2 * 1",
        Expression::mul(
            Expression::mul(Expression::value(3), Expression::value(2)),
            Expression::value(1)
        )
    )]
    #[case(
        "3 * 2 + 1",
        Expression::plus(
            Expression::mul(Expression::value(3), Expression::value(2)),
            Expression::value(1)
        )
    )]
    #[case(
        "3 - 2 / 1",
        Expression::minus(
            Expression::value(3),
            Expression::div(Expression::value(2), Expression::value(1)),
        )
    )]
    #[case(
        "(3 - 2) / 1",
        Expression::div(
            Expression::minus(Expression::value(3), Expression::value(2)),
            Expression::value(1),
        )
    )]
    #[case(
        "(3 - (2 * 1))",
        Expression::minus(
            Expression::value(3),
            Expression::mul(Expression::value(2), Expression::value(1)),
        )
    )]
    #[case(
        "((2 * 1) - 3)",
        Expression::minus(
            Expression::mul(Expression::value(2), Expression::value(1)),
            Expression::value(3),
        )
    )]
    #[case(
        "3+4 *5/( 6* 2 ) --71",
        Expression::minus(
            Expression::plus(
                Expression::value(3),
                Expression::div(
                    Expression::mul(Expression::value(4), Expression::value(5)),
                    Expression::mul(Expression::value(6), Expression::value(2))
                )
            ),
            Expression::value(-71),
        )
    )]
    fn parse_expression(#[case] input: &str, #[case] expected: Expression) {
        assert_eq!(parse(input), Ok(("", expected)));
    }

    #[rstest]
    #[case(Expression::value(1), 1)]
    #[case(Expression::plus(Expression::value(1), Expression::value(1)), 2)]
    #[case(Expression::minus(Expression::value(5), Expression::value(2)), 3)]
    #[case(Expression::div(Expression::value(12), Expression::value(3)), 4)]
    #[case(Expression::mul(Expression::value(2), Expression::value(3)), 6)]
    fn evaluate(#[case] expression: Expression, #[case] expected_result: impl Into<Decimal>) {
        assert_eq!(expression.evaluate(), Value(expected_result.into()));
    }

    #[test]
    fn into_f64() {
        let value = Value(Decimal::new(2, 0));
        assert_eq!(value.try_into_f64(), Ok(2.0));
    }

    #[test]
    fn into_f32() {
        let value = Value(Decimal::new(2, 0));
        assert_eq!(value.try_into_f32(), Ok(2.0));
    }
}
