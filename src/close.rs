use nom::{bytes::complete::tag, character::complete::space1, sequence::tuple};

use crate::{account::account, date::date, Account, Date, IResult, Span};

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
}

pub(crate) fn close(input: Span<'_>) -> IResult<'_, Close<'_>> {
    let (input, date) = date(input)?;
    let (input, _) = tuple((space1, tag("close"), space1))(input)?;
    let (input, account) = account(input)?;
    Ok((input, Close { date, account }))
}
