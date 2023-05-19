use std::collections::HashMap;

use crate::{
    account::{account, Account},
    amount::{amount, currency, Amount},
    date::{date, Date},
    string::string,
    IResult, Span,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, one_of, satisfy, space0, space1},
    combinator::{map, recognize},
    multi::{fold_many0, many0},
    sequence::{pair, preceded, separated_pair, tuple},
};

#[cfg(feature = "unstable")]
pub type Metadata<'a> = HashMap<String, Value<'a>>;

#[cfg(not(feature = "unstable"))]
pub(crate) type Metadata<'a> = HashMap<String, Value<'a>>;

#[derive(Clone, Debug, PartialEq)]
#[cfg(feature = "unstable")]
#[non_exhaustive]
pub enum Value<'a> {
    Account(Account<'a>),
    Amount(Amount<'a>),
    Currency(&'a str),
    Date(Date),
    String(String),
    // FIXME: Do not yet handle:
    // 1) Empty metadata
    // 2) Tags
    // 3) Numbers without currencies
}

#[derive(Clone, Debug, PartialEq)]
#[cfg(not(feature = "unstable"))]
#[non_exhaustive]
pub(crate) enum Value<'a> {
    Account(Account<'a>),
    Amount(Amount<'a>),
    Currency(&'a str),
    Date(Date),
    String(String),
}

fn metadata_key(input: Span<'_>) -> IResult<'_, &str> {
    map(
        recognize(pair(
            satisfy(|c: char| c.is_ascii_lowercase() && c.is_ascii_alphabetic()),
            many0(alt((
                satisfy(|c: char| c.is_ascii_alphabetic()),
                satisfy(char::is_numeric),
                one_of("-_"),
            ))),
        )),
        |s: Span<'_>| *s.fragment(),
    )(input)
}

fn metadata_value(input: Span<'_>) -> IResult<'_, Value<'_>> {
    alt((
        map(account, Value::Account),
        map(amount, Value::Amount),
        map(currency, Value::Currency),
        map(date, Value::Date),
        map(string, Value::String),
    ))(input)
}

fn metadata_line(input: Span<'_>) -> IResult<'_, (&str, Value<'_>)> {
    separated_pair(
        metadata_key,
        tuple((space0, tag(":"), space0)),
        metadata_value,
    )(input)
}

pub(crate) fn metadata(input: Span<'_>) -> IResult<'_, Metadata<'_>> {
    fold_many0(
        preceded(tuple((space0, line_ending, space1)), metadata_line),
        HashMap::new,
        |mut acc: HashMap<_, _>, (k, v)| {
            // Only the first entry is kept per Beancount documentation
            let k = k.to_string();
            let _: &mut Value<'_> = acc.entry(k).or_insert(v);
            acc
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::account::Type;

    #[test]
    fn valid_metadata() {
        let input = r#"
            abc: "hello"
            def-hij: 1 USD"#;
        let mut expected_map = HashMap::new();
        std::mem::drop(
            expected_map.insert(String::from("abc"), Value::String(String::from("hello"))),
        );
        std::mem::drop(expected_map.insert(
            String::from("def-hij"),
            Value::Amount(Amount::new(1, "USD")),
        ));
        assert_eq!(metadata(Span::new(input)).unwrap().1, expected_map);
    }

    #[test]
    fn repeated_key_ignored() {
        let input = r#"
            abc: "hello"
            abc: 1 USD"#;
        let mut expected_map = HashMap::new();
        std::mem::drop(
            expected_map.insert(String::from("abc"), Value::String(String::from("hello"))),
        );
        assert_eq!(metadata(Span::new(input)).unwrap().1, expected_map);
    }

    #[test]
    fn value_is_account() {
        let input = r#"abc: Assets:Unknown"#;
        assert_eq!(
            metadata_line(Span::new(input)).unwrap().1,
            (
                "abc",
                Value::Account(Account::new(Type::Assets, ["Unknown"]))
            )
        );
    }

    #[test]
    fn value_is_amount() {
        let input = r#"abc: 1 USD"#;
        assert_eq!(
            metadata_line(Span::new(input)).unwrap().1,
            ("abc", Value::Amount(Amount::new(1, "USD")))
        );
    }

    #[test]
    fn value_is_currency() {
        let input = r#"abc: CHF"#;
        assert_eq!(
            metadata_line(Span::new(input)).unwrap().1,
            ("abc", Value::Currency("CHF"))
        );
    }

    #[test]
    fn value_is_date() {
        let input = r#"abc: 2014-01-01"#;
        assert_eq!(
            metadata_line(Span::new(input)).unwrap().1,
            ("abc", Value::Date(Date::new(2014, 1, 1)))
        );
    }

    #[test]
    fn value_is_string() {
        let input = r#"abc: "def""#;
        assert_eq!(
            metadata_line(Span::new(input)).unwrap().1,
            ("abc", Value::String(String::from("def")))
        );
    }
}
