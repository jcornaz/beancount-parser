use crate::close::close;
use crate::open::open;
use crate::price::{price, Price};
use crate::{Close, Open};
use nom::branch::alt;
use nom::{combinator::map, IResult};

use crate::transaction::{transaction, Transaction};

/// A directive
///
/// A beancount file is made of directives.
///
/// By far the the most common directive is the [`Transaction`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Directive<'a> {
    /// The [`Transaction`](crate::Transaction) directive
    Transaction(Transaction<'a>),
    /// The [`Price`](crate::Price) directive
    Price(Price<'a>),
    /// The [`Open`](crate::Open) account directive
    Open(Open<'a>),
    /// The [`Close`](crate::Close) account directive
    Close(Close<'a>),
}

impl<'a> Directive<'a> {
    /// Returns the [`Transaction`] reference if this directive is a transaction
    ///
    /// See also [`Directive::into_transaction`]
    #[must_use]
    pub fn as_transaction(&self) -> Option<&Transaction<'a>> {
        match self {
            Directive::Transaction(trx) => Some(trx),
            _ => None,
        }
    }

    /// Convert into a [`Transaction`] if this directive is a transaction
    ///
    /// See also [`Directive::as_transaction`]
    #[must_use]
    pub fn into_transaction(self) -> Option<Transaction<'a>> {
        match self {
            Directive::Transaction(trx) => Some(trx),
            _ => None,
        }
    }
}

pub(crate) fn directive(input: &str) -> IResult<&str, Directive<'_>> {
    alt((
        map(transaction, Directive::Transaction),
        map(price, Directive::Price),
        map(open, Directive::Open),
        map(close, Directive::Close),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn transaction() {
        let input = r#"2022-09-10 txn "My transaction""#;
        let (_, directive) = directive(input).expect("should successfully parse directive");
        let transaction = directive
            .as_transaction()
            .expect("the directive should be a transaction");
        assert_eq!(transaction.narration(), Some("My transaction"));
    }

    #[rstest]
    fn invalid(
        #[values(
            "2022-09-11 txn that is incorrect",
            "2022-09-11 * that is incorrect",
            "2022-09-11 ! that is incorrect"
        )]
        input: &str,
    ) {
        assert!(matches!(directive(input), Err(nom::Err::Failure(_))));
    }

    #[rstest]
    fn not_matching(#[values(" ")] input: &str) {
        assert!(matches!(directive(input), Err(nom::Err::Error(_))));
    }
}
