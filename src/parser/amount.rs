use nom::{
    character::complete::{char, digit0, digit1, space0},
    combinator::{map, map_res, opt, recognize},
    sequence::{preceded, tuple},
    IResult,
};
use rust_decimal::Decimal;

fn operation(input: &str) -> IResult<&str, Decimal> {
    map(
        tuple((value, space0, operator, space0, value)),
        |(a, _, op, _, b)| match op {
            '/' => a / b,
            _ => unreachable!("unexpected operator {op}"),
        },
    )(input)
}

fn operator(input: &str) -> IResult<&str, char> {
    char('/')(input)
}

fn value(input: &str) -> IResult<&str, Decimal> {
    let value_string = recognize(tuple((
        opt(char('-')),
        digit0,
        opt(preceded(char('.'), digit1)),
    )));
    map_res(value_string, |s: &str| s.parse())(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod value {
        use super::*;
        use rust_decimal::Decimal;

        #[rstest]
        #[case("0", Decimal::ZERO)]
        #[case("42", Decimal::new(42, 0))]
        #[case("1.1", Decimal::new(11, 1))]
        #[case(".1", Decimal::new(1, 1))]
        #[case("-2", -Decimal::new(2, 0))]
        fn direct_value(#[case] input: &str, #[case] expected: Decimal) {
            assert_eq!(value(input), Ok(("", expected)))
        }

        #[rstest]
        #[case("3 / 2", Decimal::new(15, 1))]
        fn evaluate_operation(#[case] input: &str, #[case] expected: Decimal) {
            assert_eq!(operation(input), Ok(("", expected)))
        }
    }
}
