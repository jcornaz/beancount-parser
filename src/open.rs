use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    sequence::{preceded, separated_pair, tuple},
    IResult,
};

#[cfg(feature = "unstable")]
use crate::amount;
use crate::{account, date::date, Account, Date};
#[cfg(feature = "unstable")]
use nom::combinator::opt;

/// Open account directive
#[derive(Debug, Clone)]
pub struct Open<'a> {
    date: Date,
    account: Account<'a>,
    #[cfg(feature = "unstable")]
    currencies: Vec<&'a str>,
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
    #[cfg(feature = "unstable")]
    #[must_use]
    pub fn currencies(&self) -> &[&'a str] {
        &self.currencies
    }
}

#[cfg(not(feature = "unstable"))]
pub(crate) fn open(input: &str) -> IResult<&str, Open<'_>> {
    map(
        separated_pair(
            date,
            space1,
            preceded(tuple((tag("open"), space1)), account::account),
        ),
        |(date, account)| Open { date, account },
    )(input)
}

#[cfg(feature = "unstable")]
pub(crate) fn open(input: &str) -> IResult<&str, Open<'_>> {
    map(
        separated_pair(
            date,
            space1,
            tuple((
                preceded(tuple((tag("open"), space1)), account::account),
                opt(preceded(space1, amount::currency)),
            )),
        ),
        |(date, (account, currency))| Open {
            date,
            account,
            currencies: currency.into_iter().collect(),
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_open_directive() {
        let input = "2022-10-14 open Assets:A";
        let (_, open) = open(input).expect("should successfuly parse the input");
        assert_eq!(open.date(), Date::new(2022, 10, 14));
        assert_eq!(open.account(), &Account::new(account::Type::Assets, ["A"]));
    }
}
