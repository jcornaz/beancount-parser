use nom::{
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    multi::separated_list0,
    sequence::{preceded, tuple},
};

#[cfg(feature = "unstable")]
use crate::pest_parser::Pair;
use crate::{account, date::date, Account, Date, Span};
use crate::{amount, IResult};

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

    #[cfg(feature = "unstable")]
    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let date = Date::from_pair(inner.next().expect("no date in open directive"));
        let account = Account::from_pair(inner.next().expect("no account in open directive"));
        let currencies = inner.map(|c| c.as_str()).collect();
        Open {
            date,
            account,
            currencies,
        }
    }
}

pub(crate) fn open(input: Span<'_>) -> IResult<'_, Open<'_>> {
    let (input, date) = date(input)?;
    let (input, _) = tuple((space1, tag("open"), space1))(input)?;
    let (input, account) = account::account(input)?;
    let (input, currencies) =
        separated_list0(char(','), preceded(space0, amount::currency))(input)?;
    Ok((
        input,
        Open {
            date,
            account,
            currencies,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_open_directive() {
        let (_, open) = open(Span::new("2022-10-14 open Assets:A")).unwrap();
        assert_eq!(open.date(), Date::new(2022, 10, 14));
        assert_eq!(open.account(), &Account::new(account::Type::Assets, ["A"]));
        assert_eq!(open.currencies().len(), 0);
    }

    #[test]
    fn open_with_single_currency_constraint() {
        let (_, open) = open(Span::new(
            "2014-05-01 open Liabilities:CreditCard:CapitalOne CHF",
        ))
        .unwrap();
        assert_eq!(open.currencies(), &["CHF"]);
    }

    #[test]
    fn open_with_multiple_currency_constraints() {
        let (_, open) = open(Span::new(
            "2014-05-01 open Liabilities:CreditCard:CapitalOne CHF, USD,EUR",
        ))
        .unwrap();
        assert_eq!(open.currencies(), &["CHF", "USD", "EUR"]);
    }
}
