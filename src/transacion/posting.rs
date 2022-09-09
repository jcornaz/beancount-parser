use nom::{
    branch::alt,
    character::complete::{line_ending, space0, space1},
    combinator::{eof, map, opt},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use crate::{
    account::{account, Account},
    amount::{amount, Amount},
};

use super::{flag, Flag};

#[derive(Debug, Clone, PartialEq)]
pub struct Posting<'a> {
    flag: Option<Flag>,
    account: Account<'a>,
    amount: Option<Amount<'a>>,
}

impl<'a> Posting<'a> {
    pub fn flag(&self) -> Option<Flag> {
        self.flag
    }
}

pub fn posting(input: &str) -> IResult<&str, Posting<'_>> {
    map(
        delimited(
            space1,
            tuple((
                opt(terminated(flag, space1)),
                account,
                opt(preceded(space1, amount)),
            )),
            tuple((space0, alt((line_ending, eof)))),
        ),
        |(flag, account, amount)| Posting {
            flag,
            account,
            amount,
        },
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
    #[case("  Assets:A:B  10 CHF", Posting { account: Account::new(AccountType::Assets, ["A", "B"]), amount: Some(Amount::new(10, "CHF")), flag: None })]
    #[case("  Assets:A:B  10 CHF \n", Posting { account: Account::new(AccountType::Assets, ["A", "B"]), amount: Some(Amount::new(10, "CHF")), flag: None })]
    #[case("  Assets:A:B", Posting { account: Account::new(AccountType::Assets, ["A", "B"]), amount: None, flag: None })]
    fn parse_posting(#[case] input: &str, #[case] expected: Posting<'_>) {
        let (_, actual) = posting(input).expect("Should succesfully parse the posting");
        assert_eq!(actual, expected);
    }

    #[test]
    fn with_flag() {
        let (_, posting) =
            posting(" ! Assets:A 1 EUR").expect("should succesfully parse the posting");
        assert_eq!(posting.flag(), Some(Flag::Pending));
    }
}
