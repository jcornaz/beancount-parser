use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// Currency
///
/// One may use [`Currency::as_str`] to get the string representation of the currency
///
/// For an example, look at the [`Price`] directive
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency<'a>(Cow<'a, str>);

impl Currency<'_> {
    #[must_use]
    pub fn into_owned(self) -> Currency<'static> {
        Currency(self.0.into_owned().into())
    }
}

impl Display for Currency<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Currency<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Currency<'_> {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<'a> From<Currency<'a>> for Cow<'a, str> {
    fn from(value: Currency<'a>) -> Self {
        value.0
    }
}

impl<'a> From<Currency<'a>> for String {
    fn from(value: Currency<'a>) -> Self {
        value.0.into_owned()
    }
}

impl<'a> TryFrom<&'a str> for Currency<'a> {
    type Error = Invalid;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if is_valid(value) {
            Ok(Self(Cow::Borrowed(value)))
        } else {
            Err(Invalid)
        }
    }
}

impl FromStr for Currency<'static> {
    type Err = Invalid;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if is_valid(s) {
            Ok(Self(Cow::Owned(s.into())))
        } else {
            Err(Invalid)
        }
    }
}

/// Error returned when failing to convert a string into a currency
#[derive(Debug)]
pub struct Invalid;

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
    fn parse_valid_currency(#[case] input: &str) {
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
    fn parse_invalid_currency(#[case] input: &str) {
        Currency::from_str(input).unwrap_err();
    }
}
