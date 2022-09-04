use nom::{
    character::complete::space1,
    combinator::map,
    sequence::{preceded, separated_pair},
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
        preceded(space1, separated_pair(account, space1, amount)),
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
    fn parse_posting(#[case] input: &str, #[case] expected: Posting<'_>) {
        assert_eq!(posting(input), Ok(("", expected)));
    }
}
