use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
    str::FromStr,
};

use crate::v3::error::ParseError;

const SEP: char = ':';

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Account<'a>(Cow<'a, str>);

impl Account<'_> {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_owned(self) -> Account<'static> {
        Account(Cow::Owned(self.0.into_owned()))
    }
}

impl Account<'_> {
    pub fn type_(&self) -> AccountType {
        // Here we can unwrap, because we know the content is a valid accont name
        self.0.split(SEP).next().unwrap().parse().unwrap()
    }

    pub fn components(&self) -> impl Iterator<Item = AccountComponent<'_>> {
        self.0
            .split(SEP)
            .map(|s| AccountComponent(Cow::Borrowed(s)))
    }
}

impl Display for Account<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Account<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Account<'_> {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Account<'_>> for String {
    fn from(value: Account<'_>) -> Self {
        value.0.into_owned()
    }
}

impl<'a> TryFrom<&'a str> for Account<'a> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        parse(value).ok_or(ParseError)
    }
}

impl FromStr for Account<'static> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s).ok_or(ParseError).map(Account::into_owned)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AccountType {
    Assets,
    Liabilities,
    Equity,
    Income,
    Expenses,
}

impl AccountType {
    pub fn as_str(&self) -> &str {
        match self {
            AccountType::Assets => "Assets",
            AccountType::Liabilities => "Liabilities",
            AccountType::Equity => "Equity",
            AccountType::Income => "Income",
            AccountType::Expenses => "Expenses",
        }
    }
}

impl AsRef<str> for AccountType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AccountType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Assets" => Ok(AccountType::Assets),
            "Liabilities" => Ok(AccountType::Liabilities),
            "Equity" => Ok(AccountType::Equity),
            "Income" => Ok(AccountType::Income),
            "Expenses" => Ok(AccountType::Expenses),
            _ => Err(ParseError),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountComponent<'a>(Cow<'a, str>);

impl AccountComponent<'_> {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_owned(self) -> AccountComponent<'static> {
        AccountComponent(Cow::Owned(self.0.into_owned()))
    }
}

impl Display for AccountComponent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for AccountComponent<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

pub fn parse(input: &str) -> Option<Account<'_>> {
    if is_valid(input) {
        Some(Account(Cow::Borrowed(input)))
    } else {
        None
    }
}

/// Each component of the account names begin with a capital letter or a number and are followed by letters, numbers or dash (-) characters. All other characters are disallowed.
pub fn is_valid(input: &str) -> bool {
    let mut iter = input.split(SEP);
    let Some(type_) = iter.next() else {
        return false;
    };
    if type_ != "Assets"
        && type_ != "Liabilities"
        && type_ != "Equity"
        && type_ != "Income"
        && type_ != "Expenses"
    {
        return false;
    }
    iter.all(is_valid_component)
}

/// Each component of the account names begin with a capital letter or a number and are followed by letters, numbers or dash (-) characters. All other characters are disallowed.
pub fn is_valid_component(input: &str) -> bool {
    let mut chars = input.chars();
    let Some(first_char) = chars.next() else {
        return false;
    };
    if !first_char.is_ascii_uppercase() && !first_char.is_ascii_digit() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("Assets:US:BofA:Checking", AccountType::Assets)]
    #[case("Assets:12", AccountType::Assets)]
    #[case("Liabilities:CA:RBC:CreditCard", AccountType::Liabilities)]
    #[case("Equity:Retained-Earnings", AccountType::Equity)]
    #[case("Income:US:Acme:Salary", AccountType::Income)]
    #[case("Expenses:Food:Groceries", AccountType::Expenses)]
    #[case("Assets", AccountType::Assets)]
    fn should_parse_valid_account(#[case] input: &str, #[case] expected_type: AccountType) {
        let account: Account = input.parse().unwrap();
        assert_eq!(account.as_str(), input);
        assert_eq!(account.type_(), expected_type);
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    #[case("Something:Else")]
    #[case("Assets:Oops.")]
    #[case("Assets:lowercase")]
    fn should_not_parse_invalid_account(#[case] input: &str) {
        let _: ParseError = input.parse::<Account>().unwrap_err();
    }
}
