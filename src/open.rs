use nom::{
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    combinator::map,
    multi::separated_list0,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};

use crate::amount;
use crate::{account, date::date, Account, Date};

/// Open account directive
#[derive(Debug, Clone)]
pub struct Open<'a> {
    pub(crate) date: Date,
    pub(crate) account: Account<'a>,
    pub(crate) currencies: Vec<&'a str>,
}

impl<'a> Open<'a> {
    /// Date at which the account is open
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// Account being open
    #[must_use]
    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    /// Returns the currency constraints
    #[must_use]
    pub fn currencies(&self) -> &[&'a str] {
        &self.currencies
    }
}

pub(crate) fn open(input: &str) -> IResult<&str, Open<'_>> {
    map(
        separated_pair(
            date,
            space1,
            tuple((
                preceded(tuple((tag("open"), space1)), account::account),
                separated_list0(char(','), preceded(space0, amount::currency)),
            )),
        ),
        |(date, (account, currencies))| Open {
            date,
            account,
            currencies,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_open_directive() {
        let (_, open) = open("2022-10-14 open Assets:A").unwrap();
        assert_eq!(open.date(), Date::new(2022, 10, 14));
        assert_eq!(open.account(), &Account::new(account::Type::Assets, ["A"]));
        assert_eq!(open.currencies().len(), 0);
    }

    #[test]
    fn open_with_single_currency_constraint() {
        let (_, open) = open("2014-05-01 open Liabilities:CreditCard:CapitalOne CHF").unwrap();
        assert_eq!(open.currencies(), &["CHF"]);
    }

    #[test]
    fn open_with_multiple_currency_constraints() {
        let (_, open) =
            open("2014-05-01 open Liabilities:CreditCard:CapitalOne CHF, USD,EUR").unwrap();
        assert_eq!(open.currencies(), &["CHF", "USD", "EUR"]);
    }
}
