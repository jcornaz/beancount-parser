use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    sequence::{separated_pair, tuple},
};

use crate::{account::account, date::date, Account, Date, IResult};

#[cfg(feature = "unstable")]
use crate::pest_parser::Pair;

/// The close account directive
#[derive(Debug, Clone)]
pub struct Close<'a> {
    pub(crate) date: Date,
    pub(crate) account: Account<'a>,
}

impl<'a> Close<'a> {
    /// The date at which the account is closed
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// Account being closed
    #[must_use]
    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    #[cfg(feature = "unstable")]
    pub(crate) fn from_pair(pair: Pair<'_>) -> Close<'_> {
        let mut inner = pair.into_inner();
        let date = Date::from_pair(inner.next().expect("no date in close directive"));
        let account = Account::from_pair(inner.next().expect("no account in close directive"));
        Close { date, account }
    }
}

pub(crate) fn close(input: &str) -> IResult<'_, Close<'_>> {
    map(
        separated_pair(date, tuple((space1, tag("close"), space1)), account),
        |(date, account)| Close { date, account },
    )(input)
}
