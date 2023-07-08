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
    sync::Arc,
};

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{char, satisfy, space1},
    combinator::{iterator, recognize},
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

pub(crate) fn parse<D: Decimal>(input: Span<'_>) -> IResult<'_, Map<D>> {
    let mut iter = iterator(input, alt((entry.map(Some), empty_line.map(|_| None))));
    let map: HashMap<_, _> = iter.flatten().collect();
    let (input, _) = iter.finish()?;
    Ok((input, map))
}

fn entry<D: Decimal>(input: Span<'_>) -> IResult<'_, (Key, Value<D>)> {
    let (input, _) = space1(input)?;
    let (input, key) = recognize(preceded(
        satisfy(char::is_lowercase),
        take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
    ))(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = alt((
        string.map(ToOwned::to_owned).map(Value::String),
        amount::expression.map(Value::Number),
        amount::currency.map(Value::Currency),
    ))(input)?;
    let (input, _) = end_of_line(input)?;
    Ok((input, (Key((*key.fragment()).into()), value)))
}
