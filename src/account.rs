use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Account<'a> {
    type_: Type,
    components: Vec<&'a str>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type {
    Assets,
    Liabilities,
    Equity,
    Income,
    Expenses,
}

impl<'a> Account<'a> {
    pub(crate) fn new(type_: Type, path: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            type_,
            components: path.into_iter().collect(),
        }
    }
}

pub(crate) fn account(input: &str) -> IResult<&str, Account<'_>> {
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

fn type_(input: &str) -> IResult<&str, Type> {
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
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Assets:MyAccount", Account::new(Type::Assets, ["MyAccount"]))]
    #[case("Liabilities:A:B:C", Account::new(Type::Liabilities, ["A", "B", "C"]))]
    #[case("Income:Foo:Bar12", Account::new(Type::Income, ["Foo", "Bar12"]))]
    #[case("Expenses:3Foo", Account::new(Type::Expenses, ["3Foo"]))]
    #[case("Equity:Foo-Bar", Account::new(Type::Equity, ["Foo-Bar"]))]
    fn valid_account(#[case] input: &str, #[case] expected: Account<'_>) {
        assert_eq!(account(input), Ok(("", expected)))
    }
}
