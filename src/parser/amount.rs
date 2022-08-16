use nom::{
    character::complete::{char, digit0, digit1},
    combinator::{map, map_res, opt, recognize},
    sequence::{preceded, tuple},
    IResult,
};

use crate::Value;

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

    mod value {
        use super::*;
        use rust_decimal::Decimal;

        #[rstest]
        #[case("0", Value(Decimal::ZERO))]
        #[case("42", Value(Decimal::new(42, 0)))]
        #[case("1.1", Value(Decimal::new(11, 1)))]
        #[case(".1", Value(Decimal::new(1, 1)))]
        #[case("-2", Value(-Decimal::new(2, 0)))]
        fn direct_value(#[case] input: &str, #[case] expected: Value) {
            assert_eq!(value(input), Ok(("", expected)))
        }
    }
}
