use crate::{Account, Date};

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
