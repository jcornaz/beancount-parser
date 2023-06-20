use nom::{
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    multi::separated_list0,
    sequence::tuple,
};

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
}

pub(crate) fn directive(input: Span<'_>) -> IResult<'_, Open<'_>> {
    let (input, date) = date(input)?;
    let (input, _) = tuple((space1, tag("open"), space1))(input)?;
    content(date)(input)
}

pub(crate) fn content(date: Date) -> impl FnMut(Span<'_>) -> IResult<'_, Open<'_>> {
    move |input| {
        let (input, account) = account::account(input)?;
        let (input, _) = space0(input)?;
        let separator = tuple((space0, char(','), space0));
        let (input, currencies) = separated_list0(separator, amount::currency)(input)?;
        Ok((
            input,
            Open {
                date,
                account,
                currencies,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_open_directive() {
        let (_, open) = directive(Span::new("2022-10-14 open Assets:A")).unwrap();
        assert_eq!(open.date(), Date::new(2022, 10, 14));
        assert_eq!(open.account(), &Account::new(account::Type::Assets, ["A"]));
        assert_eq!(open.currencies().len(), 0);
    }

    #[test]
    fn open_with_single_currency_constraint() {
        let (_, open) = directive(Span::new(
            "2014-05-01 open Liabilities:CreditCard:CapitalOne CHF",
        ))
        .unwrap();
        assert_eq!(open.currencies(), &["CHF"]);
    }

    #[rstest]
    fn open_with_multiple_currency_constraints(
        #[values(
            "2014-05-01 open Liabilities:CreditCard:CapitalOne CHF, USD,EUR",
            "2014-05-01 open Liabilities:CreditCard:CapitalOne CHF ,\tUSD  ,EUR"
        )]
        input: &str,
    ) {
        let (_, open) = directive(Span::new(input)).unwrap();
        assert_eq!(open.currencies(), &["CHF", "USD", "EUR"]);
    }
}
