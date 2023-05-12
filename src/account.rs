//! Types for representing an [`Account`]

use std::fmt::Display;

#[cfg(feature = "unstable")]
use crate::pest_parser::Pair;
use crate::IResult;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
};

/// Account
///
/// An account has a type (`Assets`, `Liabilities`, `Equity`, `Income` or `Expenses`)
/// and components.
///
/// # Examples
///
/// * `Assets:Liquidity:Cash` (type: `Assets`, components: ["Liquidity", "Cash"]
/// * `Expenses:Groceries` (type: `Assets`, components: ["Groceries"]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Account<'a> {
    type_: Type,
    components: Vec<&'a str>,
}

impl Display for Account<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_)?;
        for c in &self.components {
            write!(f, ":{c}")?;
        }
        Ok(())
    }
}

/// Type of account
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    /// The assets
    Assets,
    /// The liabilities
    Liabilities,
    /// The equity
    Equity,
    /// Income
    Income,
    /// Expenses
    Expenses,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<'a> Account<'a> {
    pub(crate) fn new(type_: Type, path: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            type_,
            components: path.into_iter().collect(),
        }
    }

    /// Returns the type of account
    #[must_use]
    pub fn type_(&self) -> Type {
        self.type_
    }

    /// Returns the components
    #[must_use]
    pub fn components(&self) -> &[&'a str] {
        self.components.as_ref()
    }

    #[cfg(feature = "unstable")]
    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let type_ = match inner.next().expect("no account type in account").as_str() {
            "Assets" => Type::Assets,
            "Liabilities" => Type::Liabilities,
            "Expenses" => Type::Expenses,
            "Income" => Type::Income,
            "Equity" => Type::Equity,
            _ => unreachable!("invalid account type"),
        };
        let components = inner.map(|c| c.as_str()).collect();
        Account { type_, components }
    }
}

pub(crate) fn account(input: &str) -> IResult<'_, Account<'_>> {
    map(
        separated_pair(
            type_,
            char(':'),
            separated_list1(
                char(':'),
                take_while1(|c: char| c.is_alphanumeric() || c == '-'),
            ),
        ),
        |(t, p)| Account::new(t, p),
    )(input)
}

fn type_(input: &str) -> IResult<'_, Type> {
    alt((
        map(tag("Assets"), |_| Type::Assets),
        map(tag("Liabilities"), |_| Type::Liabilities),
        map(tag("Income"), |_| Type::Income),
        map(tag("Expenses"), |_| Type::Expenses),
        map(tag("Equity"), |_| Type::Equity),
    ))(input)
}

#[cfg(test)]
mod tests {
    use nom::combinator::all_consuming;

    use super::*;

    #[rstest]
    #[case("Assets:MyAccount", Account::new(Type::Assets, ["MyAccount"]))]
    #[case("Liabilities:A:B:C", Account::new(Type::Liabilities, ["A", "B", "C"]))]
    #[case("Income:Foo:Bar12", Account::new(Type::Income, ["Foo", "Bar12"]))]
    #[case("Expenses:3Foo", Account::new(Type::Expenses, ["3Foo"]))]
    #[case("Equity:Foo-Bar", Account::new(Type::Equity, ["Foo-Bar"]))]
    fn valid_account(#[case] input: &str, #[case] expected: Account<'_>) {
        let (_, actual) = all_consuming(account)(input).unwrap();
        assert_eq!(actual, expected);
        let formatted = format!("{actual}");
        assert_eq!(&formatted, input);
    }

    #[rstest]
    #[case(Type::Assets, "Assets")]
    #[case(Type::Liabilities, "Liabilities")]
    #[case(Type::Income, "Income")]
    #[case(Type::Expenses, "Expenses")]
    #[case(Type::Equity, "Equity")]
    fn display_type(#[case] type_: Type, #[case] expected: &str) {
        let actual = format!("{type_}");
        assert_eq!(actual, expected);
    }
}
