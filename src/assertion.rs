use crate::account::{account, Account};
use crate::date::date;
use crate::Date;
use crate::{amount::amount, Amount};

use nom::{
    bytes::complete::tag,
    character::streaming::space1,
    combinator::map,
    sequence::{terminated, tuple},
    IResult,
};

/// Account balance assertion directive
#[derive(Clone, Debug, PartialEq)]
pub struct Assertion<'a> {
    date: Date,
    account: Account<'a>,
    amount: Amount<'a>,
}

impl<'a> Assertion<'a> {
    /// Date at which the assertion is calculated
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// Account to test
    #[must_use]
    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    /// Expected balance amount
    #[must_use]
    pub fn amount(&self) -> &Amount<'a> {
        &self.amount
    }
}

pub(crate) fn assertion(input: &str) -> IResult<&str, Assertion<'_>> {
    map(
        tuple((
            terminated(date, tuple((space1, tag("balance"), space1))),
            terminated(account, space1),
            amount,
        )),
        |(date, account, amount)| Assertion {
            date,
            account,
            amount,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use nom::combinator::all_consuming;

    use crate::account::Type;

    #[test]
    fn valid_assertion() {
        let input = "2014-01-01 balance Assets:Unknown 1 USD";
        let r = all_consuming(assertion)(input);
        assert_eq!(
            r,
            Ok((
                "",
                Assertion {
                    date: Date::new(2014, 1, 1),
                    account: Account::new(Type::Assets, ["Unknown"]),
                    amount: Amount::new(1, "USD")
                }
            ))
        );
    }

    #[test]
    fn invalid_assertion() {
        let input = "2014-01-01 balance";
        let p = all_consuming(assertion)(input);
        assert!(p.is_err());
    }
}
