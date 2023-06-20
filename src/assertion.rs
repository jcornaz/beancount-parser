use nom::{
    bytes::complete::tag,
    character::streaming::space1,
    sequence::{terminated, tuple},
};

use crate::Date;
use crate::{
    account::{account, Account},
    IResult,
};
use crate::{amount::amount, Amount};
use crate::{date::date, Span};

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

pub(crate) fn assertion(input: Span<'_>) -> IResult<'_, Assertion<'_>> {
    let (input, date) = terminated(date, tuple((space1, tag("balance"), space1)))(input)?;
    let (input, account) = terminated(account, space1)(input)?;
    let (input, amount) = amount(input)?;
    Ok((
        input,
        Assertion {
            date,
            account,
            amount,
        },
    ))
}

#[cfg(test)]
mod tests {
    use nom::combinator::all_consuming;

    use crate::account::Type;

    use super::*;

    #[test]
    fn valid_assertion() {
        let input = "2014-01-01 balance Assets:Unknown 1 USD";
        let (_, r) = all_consuming(assertion)(Span::new(input)).unwrap();
        assert_eq!(
            r,
            Assertion {
                date: Date::new(2014, 1, 1),
                account: Account::new(Type::Assets, ["Unknown"]),
                amount: Amount::new(1, "USD")
            }
        );
    }

    #[test]
    fn invalid_assertion() {
        let input = "2014-01-01 balance";
        let p = all_consuming(assertion)(Span::new(input));
        assert!(p.is_err());
    }
}
