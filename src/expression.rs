use nom::{
    branch::alt,
    character::complete::{char, digit0, digit1, space0},
    combinator::{map, map_res, opt, recognize},
    sequence::{preceded, tuple},
    IResult,
};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Expression {
    Value(Value),
    Operation(Operation),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct Value(Decimal);

#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    operator: Operator,
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum Operator {
    Divide,
    Multiply,
    Add,
    Minus,
}

impl Expression {
    fn value(dec: impl Into<Decimal>) -> Self {
        Self::Value(Value(dec.into()))
    }

    fn operation(operator: Operator, left: Self, right: Self) -> Self {
        Self::Operation(Operation {
            operator,
            left: left.into(),
            right: right.into(),
        })
    }

    fn div(left: Self, right: Self) -> Self {
        Self::operation(Operator::Divide, left, right)
    }

    fn mul(left: Self, right: Self) -> Self {
        Self::operation(Operator::Multiply, left, right)
    }

    fn plus(left: Self, right: Self) -> Self {
        Self::operation(Operator::Add, left, right)
    }

    fn minus(left: Self, right: Self) -> Self {
        Self::operation(Operator::Minus, left, right)
    }
}

fn expression(input: &str) -> IResult<&str, Expression> {
    alt((operation, map(value, Expression::Value)))(input)
}

fn operator(input: &str) -> IResult<&str, Operator> {
    alt((
        map(char('/'), |_| Operator::Divide),
        map(char('*'), |_| Operator::Multiply),
        map(char('+'), |_| Operator::Add),
        map(char('-'), |_| Operator::Minus),
    ))(input)
}

fn operation(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            map(value, Expression::Value),
            space0,
            operator,
            space0,
            expression,
        )),
        |(left, _, operator, _, right)| Expression::operation(operator, left, right),
    )(input)
}

fn value(input: &str) -> IResult<&str, Value> {
    let value_string = recognize(tuple((
        opt(char('-')),
        digit0,
        opt(preceded(char('.'), digit1)),
    )));
    map(map_res(value_string, |s: &str| s.parse()), Value)(input)
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
    fn direct_value(#[case] input: &str, #[case] expected: Decimal) {
        assert_eq!(
            expression(input),
            Ok(("", Expression::Value(Value(expected))))
        )
    }

    #[rstest]
    #[case("3 / 2", Expression::div(Expression::value(3), Expression::value(2)))]
    #[case("3 * 2", Expression::mul(Expression::value(3), Expression::value(2)))]
    #[case("3 + 2", Expression::plus(Expression::value(3), Expression::value(2)))]
    #[case("3 - 2", Expression::minus(Expression::value(3), Expression::value(2)))]
    fn simple_operation(#[case] input: &str, #[case] expected: Expression) {
        assert_eq!(expression(input), Ok(("", expected)))
    }
}
