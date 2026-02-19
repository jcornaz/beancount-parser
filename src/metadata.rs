//! Types to represent [beancount metadata](https://beancount.github.io/docs/beancount_language_syntax.html#metadata)
//!
//! # Example
//!
//! ```
//! # use beancount_parser::BeancountFile;
//! use beancount_parser::metadata::Value;
//! let input = r#"
//! 2023-05-27 commodity CHF
//!     title: "Swiss Franc"
//! "#;
//! let beancount: BeancountFile<f64> = input.parse().unwrap();
//! let directive_metadata = &beancount.directives[0].metadata;
//! assert_eq!(directive_metadata.get("title"), Some(&Value::String("Swiss Franc".into())));
//! ```

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    str::FromStr,
    sync::Arc,
};

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{char, satisfy, space1},
    combinator::{all_consuming, iterator, map, recognize},
    sequence::preceded,
    Parser,
};

use crate::{amount, empty_line, end_of_line, string, Currency, Decimal, IResult, Span};

/// Metadata map
///
/// See the [`metadata`](crate::metadata) module for an example
pub type Map<D> = HashMap<Key, Value<D>>;

/// Metadata key
///
/// See the [`metadata`](crate::metadata) module for an example
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Key(Arc<str>);

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl FromStr for Key {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let span = Span::new(s);
        match all_consuming(key).parse(span) {
            Ok((_, key)) => Ok(key),
            Err(_) => Err(crate::Error::new(s, span)),
        }
    }
}

/// Metadata value
///
/// See the [`metadata`](crate::metadata) module for an example
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Value<D> {
    /// String value
    String(String),
    /// A number or number expression
    Number(D),
    /// A [`Currency`]
    Currency(Currency),
}

impl<D: Display> Display for Value<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(value) => write!(f, r#""{value}""#),
            Value::Number(value) => Display::fmt(value, f),
            Value::Currency(value) => Display::fmt(value, f),
        }
    }
}

pub(crate) fn parse<D: Decimal>(input: Span<'_>) -> IResult<'_, Map<D>> {
    let mut iter = iterator(input, alt((entry.map(Some), empty_line.map(|()| None))));
    let map: HashMap<_, _> = iter.by_ref().flatten().collect();
    let (input, ()) = iter.finish()?;
    Ok((input, map))
}

fn entry<D: Decimal>(input: Span<'_>) -> IResult<'_, (Key, Value<D>)> {
    let (input, _) = space1(input)?;
    let (input, key) = key(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = alt((
        string.map(Value::String),
        amount::expression.map(Value::Number),
        amount::currency.map(Value::Currency),
    ))
    .parse(input)?;
    let (input, ()) = end_of_line(input)?;
    Ok((input, (key, value)))
}

fn key(input: Span<'_>) -> IResult<'_, Key> {
    map(
        recognize(preceded(
            satisfy(char::is_lowercase),
            take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
        )),
        |s: Span<'_>| Key((*s.fragment()).into()),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    fn key_from_str_should_parse_key() {
        let key: Key = "foo".parse().unwrap();
        assert_eq!(key.as_ref(), "foo");
    }

    #[rstest]
    fn key_from_str_should_not_parse_invalid_key() {
        let key: Result<Key, _> = "foo bar".parse();
        assert!(key.is_err(), "{key:?}");
    }
}
