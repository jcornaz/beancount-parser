use nom::{
    character::complete::space1,
    combinator::{map, opt},
    sequence::{preceded, terminated, tuple},
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

    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    pub fn amount(&self) -> Option<&Amount<'a>> {
        self.amount.as_ref()
    }
}

pub fn posting(input: &str) -> IResult<&str, Posting<'_>> {
    map(
        tuple((
            opt(terminated(flag, space1)),
            account,
            opt(preceded(space1, amount)),
        )),
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

    #[test]
    fn simple_posting() {
        let input = "Assets:A:B 10 CHF";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(
            posting.account(),
            &Account::new(AccountType::Assets, ["A", "B"])
        );
        assert_eq!(posting.amount(), Some(&Amount::new(10, "CHF")));
    }

    #[test]
    fn without_amount() {
        let input = "Assets:A:B";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert!(posting.amount().is_none());
    }

    #[test]
    fn with_flag() {
        let (_, posting) =
            posting("! Assets:A 1 EUR").expect("should succesfully parse the posting");
        assert_eq!(posting.flag(), Some(Flag::Pending));
    }
}
