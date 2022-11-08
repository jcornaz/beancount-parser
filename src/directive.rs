use crate::close::close;
use crate::open::open;
use crate::price::{price, Price};
use crate::{Close, Date, Open};
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

    /// Returns the date of the directive (if there is one)
    #[must_use]
    pub fn date(&self) -> Option<Date> {
        Some(match self {
            Directive::Transaction(t) => t.date(),
            Directive::Open(o) => o.date(),
            Directive::Close(c) => c.date(),
            Directive::Price(p) => p.date(),
        })
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
    use approx::assert_ulps_eq;
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

    #[test]
    fn price() {
        let input = "2014-07-09 price CHF  5 PLN";
        let (_, directive) = directive(input).expect("should successfully parse directive");
        let Directive::Price(price) = directive else { panic!("Unexpected directive type: {directive:?}") };
        assert_eq!(price.date(), Date::new(2014, 7, 9));
        assert_eq!(price.commodity(), "CHF");
        assert_ulps_eq!(price.price().value().try_into_f64().unwrap(), 5.0);
        assert_eq!(price.price().currency(), "PLN");
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

    #[rstest]
    #[case("2022-11-06 txn", Date::new(2022, 11, 6))]
    #[case("2021-02-26 open Liabilities:Debt", Date::new(2021, 2, 26))]
    #[case("2021-02-26 close Liabilities:Debt", Date::new(2021, 2, 26))]
    #[case("2014-07-09 price HOOL  600 USD", Date::new(2014, 7, 9))]
    fn date(#[case] input: &str, #[case] expected_date: Date) {
        let (_, directive) = directive(input).expect("should successfully parse directive");
        let date = directive.date().expect("directive should have a date");
        assert_eq!(date, expected_date);
    }
}
