#![cfg(feature = "unstable")]
#![allow(unused)]

use super::{Posting, Transaction};
use crate::Amount;

/// A [`Transaction`] that has an amount for each of its postings
#[allow(clippy::module_name_repetitions)]
pub type BalancedTransaction<'a> = Transaction<'a, Amount<'a>>;

/// A [`Posting`] that is guaranteed to have an amount for each of its postings
#[allow(clippy::module_name_repetitions)]
pub type BalancedPosting<'a> = Posting<'a, Amount<'a>>;

impl<'a> Transaction<'a> {
    /// Returned the balanced version of this transaction if possible
    ///
    /// Returns `None` if the transaction cannot be balanced
    #[must_use]
    pub fn balanced(self) -> Option<BalancedTransaction<'a>> {
        let mut postings = Vec::with_capacity(self.postings.len());
        for posting in self.postings {
            let Some(amount) = posting.amount else { return None };
            postings.push(BalancedPosting {
                info: posting.info,
                amount,
            });
        }
        Some(BalancedTransaction {
            info: self.info,
            postings,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{transaction::Info, Date};
    use rstest::{fixture, rstest};

    use super::*;

    #[rstest]
    fn balance_an_emtpy_transaction(trx: Transaction<'_>) {
        let balanced: BalancedTransaction<'_> = trx.balanced().unwrap();
        assert_eq!(balanced.postings.len(), 0);
    }

    #[fixture]
    fn trx(trx_info: Info<'static>) -> Transaction<'static> {
        Transaction {
            info: trx_info,
            postings: Vec::new(),
        }
    }

    #[fixture]
    fn trx_info() -> Info<'static> {
        Info {
            date: Date::new(2022, 11, 20),
            flag: None,
            payee: None,
            narration: None,
            comment: None,
            tags: Vec::new(),
        }
    }
}
