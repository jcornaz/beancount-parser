use nom::{
    branch::alt,
    character::complete::{char, digit0, digit1, space0},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded, tuple},
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
    fn parse_value(#[case] input: &str, #[case] expected: Decimal) {
        assert_eq!(parse(input), Ok(("", Expression::Value(Value(expected)))))
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
        assert_eq!(parse(input), Ok(("", expected)))
    }
}
