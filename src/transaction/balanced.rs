#![cfg(feature = "unstable")]

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
    use crate::{
        account, transaction::posting::Info as PostingInfo, transaction::Info as TrxInfo, Account,
        Date,
    };

    use super::*;

    #[test]
    fn balance_an_emtpy_transaction() {
        let balanced: BalancedTransaction<'_> = transaction([]).balanced().unwrap();
        assert_eq!(balanced.postings.len(), 0);
    }

    #[test]
    #[ignore = "not implemeted"]
    fn simple_balance() {
        let balanced = transaction([
            posting(
                Account::new(account::Type::Assets, []),
                Some(Amount::new(1, "CHF")),
            ),
            posting(Account::new(account::Type::Income, []), None),
        ])
        .balanced()
        .unwrap();
        assert_eq!(balanced.postings.len(), 2);
        assert_eq!(balanced.postings[0].amount, Amount::new(-1, "CHF"));
        assert_eq!(balanced.postings[1].amount, Amount::new(-1, "CHF"));
    }

    fn transaction<'a>(postings: impl IntoIterator<Item = Posting<'a>>) -> Transaction<'a> {
        Transaction {
            info: TrxInfo {
                date: Date::new(2022, 11, 20),
                flag: None,
                payee: None,
                narration: None,
                comment: None,
                tags: Vec::new(),
            },
            postings: postings.into_iter().collect(),
        }
    }

    fn posting<'a>(account: Account<'a>, amount: Option<Amount<'a>>) -> Posting<'a> {
        Posting {
            info: PostingInfo {
                flag: None,
                account,
                price: None,
                cost: None,
                comment: None,
            },
            amount,
        }
    }
}
