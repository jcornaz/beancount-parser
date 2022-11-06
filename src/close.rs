use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    sequence::{separated_pair, tuple},
    IResult,
};

use crate::{account::account, date::date, Account, Date};

/// The close account directive
#[derive(Debug, Clone)]
pub struct Close<'a> {
    date: Date,
    account: Account<'a>,
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
}

pub(crate) fn close(input: &str) -> IResult<&str, Close<'_>> {
    map(
        separated_pair(date, tuple((space1, tag("close"), space1)), account),
        |(date, account)| Close { date, account },
    )(input)
}
