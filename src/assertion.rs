use nom::{
    bytes::complete::tag,
    character::streaming::space1,
    combinator::map,
    sequence::{terminated, tuple},
};

use crate::date::date;
#[cfg(feature = "unstable")]
use crate::pest_parser::Pair;
use crate::Date;
use crate::{
    account::{account, Account},
    IResult,
};
use crate::{amount::amount, Amount};

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

    #[cfg(feature = "unstable")]
    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let date = Date::from_pair(inner.next().expect("no date in balance assertion"));
        let account = Account::from_pair(inner.next().expect("no account in balance assertion"));
        let amount = Amount::from_pair(inner.next().expect("no amount in balance assertion"));
        Self {
            date,
            account,
            amount,
        }
    }
}

pub(crate) fn assertion(input: &str) -> IResult<'_, Assertion<'_>> {
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
    use nom::combinator::all_consuming;

    use crate::account::Type;

    use super::*;

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
