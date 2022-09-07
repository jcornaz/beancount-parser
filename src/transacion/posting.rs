use nom::{
    branch::alt,
    character::complete::{line_ending, space0, space1},
    combinator::{eof, map},
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

use crate::{
    account::{account, Account},
    amount::{amount, Amount},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Posting<'a> {
    account: Account<'a>,
    amount: Amount<'a>,
}

pub fn posting(input: &str) -> IResult<&str, Posting<'_>> {
    map(
        delimited(
            space1,
            separated_pair(account, space1, amount),
            tuple((space0, alt((line_ending, eof)))),
        ),
        |(account, amount)| Posting { account, amount },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Type as AccountType;
    use rstest::rstest;

    #[rstest]
    fn invalid(#[values("")] input: &str) {
        assert!(posting(input).is_err());
    }

    #[rstest]
    #[case("  Assets:A:B  10 CHF", Posting { account: Account::new(AccountType::Assets, ["A", "B"]), amount: Amount::new(10, "CHF")})]
    #[case("  Assets:A:B  10 CHF \n", Posting { account: Account::new(AccountType::Assets, ["A", "B"]), amount: Amount::new(10, "CHF")})]
    fn parse_posting(#[case] input: &str, #[case] expected: Posting<'_>) {
        assert_eq!(posting(input), Ok(("", expected)));
    }
}
