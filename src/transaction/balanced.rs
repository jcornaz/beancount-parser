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
                flag: posting.flag,
                account: posting.account,
                amount,
                price: posting.price,
                cost: posting.cost,
                comment: posting.comment,
            });
        }
        Some(BalancedTransaction {
            date: self.date,
            flag: self.flag,
            payee: self.payee,
            narration: self.narration,
            tags: self.tags,
            comment: self.comment,
            postings,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Date;
    use rstest::{fixture, rstest};

    use super::*;

    #[rstest]
    fn balance_an_emtpy_transaction(trx: Transaction<'_>) {
        let balanced: BalancedTransaction<'_> = trx.balanced().unwrap();
        assert_eq!(balanced.postings.len(), 0);
    }

    #[fixture]
    fn trx() -> Transaction<'static> {
        Transaction {
            date: Date::new(2022, 11, 20),
            flag: None,
            payee: None,
            narration: None,
            comment: None,
            tags: Vec::new(),
            postings: Vec::new(),
        }
    }
}
