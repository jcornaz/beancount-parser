use std::collections::HashMap;

use crate::{
    account::{account, Account},
    amount::{amount, currency, Amount},
    date::{date, Date},
    string::string,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, one_of, satisfy, space0, space1},
    combinator::{map, recognize},
    multi::{fold_many0, many0},
    sequence::{pair, preceded, separated_pair, tuple},
    IResult,
};

pub type Metadata<'a> = HashMap<String, MetadataValue<'a>>;

#[derive(Clone, Debug, PartialEq)]
pub enum MetadataValue<'a> {
    Account(Account<'a>),
    Amount(Amount<'a>),
    Currency(String),
    Date(Date),
    String(String),
    // FIXME: Do not yet handle:
    // 1) Empty metadata
    // 2) Tags
    // 3) Numbers without currencies
}

fn metadata_key(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        satisfy(|c: char| c.is_ascii_lowercase() && c.is_ascii_alphabetic()),
        many0(alt((
            satisfy(|c: char| c.is_ascii_alphabetic()),
            satisfy(|c: char| c.is_numeric()),
            one_of("-_"),
        ))),
    ))(input)
}

fn metadata_value(input: &str) -> IResult<&str, MetadataValue<'_>> {
    alt((
        map(account, |a| MetadataValue::Account(a)),
        map(amount, |a| MetadataValue::Amount(a)),
        map(currency, |c| MetadataValue::Currency(c.to_owned())),
        map(date, |d| MetadataValue::Date(d)),
        map(string, |s| MetadataValue::String(s)),
    ))(input)
}

fn metadata_line(input: &str) -> IResult<&str, (&str, MetadataValue<'_>)> {
    separated_pair(
        metadata_key,
        tuple((space0, tag(":"), space0)),
        metadata_value,
    )(input)
}

pub fn metadata(input: &str) -> IResult<&str, Metadata<'_>> {
    fold_many0(
        preceded(tuple((space0, line_ending, space1)), metadata_line),
        HashMap::new,
        |mut acc: HashMap<_, _>, (k, v)| {
            // Only the first entry is kept per Beancount documentation
            let k = k.to_string();
            if !acc.contains_key(&k) {
                let _ = acc.insert(k, v);
            }
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
        let _ = expected_map.insert(
            String::from("abc"),
            MetadataValue::String(String::from("hello")),
        );
        let _ = expected_map.insert(
            String::from("def-hij"),
            MetadataValue::Amount(Amount::new(1, "USD")),
        );
        assert_eq!(metadata(input), Ok(("", expected_map)));
    }

    #[test]
    fn value_is_account() {
        let input = r#"abc: Assets:Unknown"#;
        assert_eq!(
            metadata_line(input),
            Ok((
                "",
                (
                    "abc",
                    MetadataValue::Account(Account::new(Type::Assets, ["Unknown"]))
                )
            ))
        );
    }

    #[test]
    fn value_is_amount() {
        let input = r#"abc: 1 USD"#;
        assert_eq!(
            metadata_line(input),
            Ok(("", ("abc", MetadataValue::Amount(Amount::new(1, "USD")))))
        );
    }

    #[test]
    fn value_is_currency() {
        let input = r#"abc: CHF"#;
        assert_eq!(
            metadata_line(input),
            Ok(("", ("abc", MetadataValue::Currency(String::from("CHF")))))
        );
    }

    #[test]
    fn value_is_date() {
        let input = r#"abc: 2014-01-01"#;
        assert_eq!(
            metadata_line(input),
            Ok(("", ("abc", MetadataValue::Date(Date::new(2014, 1, 1)))))
        );
    }

    #[test]
    fn value_is_string() {
        let input = r#"abc: "def""#;
        assert_eq!(
            metadata_line(input),
            Ok(("", ("abc", MetadataValue::String(String::from("def")))))
        );
    }
}
