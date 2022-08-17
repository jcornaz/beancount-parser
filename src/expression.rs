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
    Operation {
        operator: Operator,
        left_operand: Box<Expression>,
        right_operand: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct Value(Decimal);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum Operator {
    Divide,
    Multiply,
    Add,
    Minus,
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
        |(left, _, operator, _, right)| Expression::Operation {
            operator,
            left_operand: left.into(),
            right_operand: right.into(),
        },
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
    #[case('/', Operator::Divide)]
    #[case('*', Operator::Multiply)]
    #[case('+', Operator::Add)]
    #[case('-', Operator::Minus)]
    fn simple_operation(#[case] op_char: char, #[case] operator: Operator) {
        let input = format!("3 {op_char} 2");
        let expected = Expression::Operation {
            operator,
            left_operand: Expression::Value(Value(Decimal::new(3, 0))).into(),
            right_operand: Expression::Value(Value(Decimal::new(2, 0))).into(),
        };
        assert_eq!(expression(&input), Ok(("", expected)))
    }
}
