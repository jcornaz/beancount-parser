use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use crate::v3::error::ParseError;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency<'a>(Cow<'a, str>);

impl Currency<'_> {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_owned(self) -> Currency<'static> {
        Currency(Cow::Owned(self.0.into_owned()))
    }
}

impl Display for Currency<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Currency<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Currency<'_> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> From<Currency<'a>> for String {
    fn from(value: Currency<'a>) -> Self {
        value.0.into_owned()
    }
}

impl<'a> TryFrom<&'a str> for Currency<'a> {
    type Error = ParseError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        parse(value).ok_or(ParseError)
    }
}

impl FromStr for Currency<'static> {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s).ok_or(ParseError).map(Currency::into_owned)
    }
}

#[must_use]
pub fn parse(input: &str) -> Option<Currency<'_>> {
    if is_valid(input) {
        Some(Currency(Cow::Owned(input.to_owned())))
    } else {
        None
    }
}

/// Returns `true` only if the `currency` is a valid currency according to beancount syntax rules.
///
/// see: <https://beancount.github.io/docs/beancount_language_syntax/#commodities-currencies>
#[must_use]
pub fn is_valid(currency: &str) -> bool {
    if currency.len() > 24 {
        return false;
    }
    let mut chars = currency.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    let mut last = first;
    for c in chars {
        if !c.is_ascii_digit() && !c.is_ascii_uppercase() && c != '_' && c != '-' && c != '.' {
            return false;
        }
        last = c;
    }
    last.is_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("CHF")]
    #[case("A")]
    #[case("THIS_IS-ALSO.42..VALID")]
    fn should_parse_valid_currency(#[case] input: &str) {
        let currency: Currency = input.parse().unwrap();
        assert_eq!(currency.as_ref(), input);
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    #[case("CHF_")]
    #[case("_CHF")]
    #[case("oops")]
    #[case("oPP")]
    #[case("OpS")]
    #[case("THIS IS NOT VALID")]
    #[case("1A")]
    #[case("A1")]
    #[case("IT_CANNOT_BE_MORE_THAN_24_CHARACTER")]
    fn should_fail_to_parse_invalid_currency(#[case] input: &str) {
        Currency::from_str(input).unwrap_err();
    }
}
