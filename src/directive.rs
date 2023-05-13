use crate::assertion::assertion;
use crate::close::close;
use crate::include::{include, Include};
use crate::open::open;
use crate::pad::{pad, Pad};
use crate::price::{price, Price};
#[cfg(feature = "unstable")]
use crate::{
    pest_parser::{Pair, Rule},
    {Commodity, Event},
};
use crate::{Assertion, Close, Date, IResult, Open, Span};
use nom::branch::alt;
use nom::combinator::map;

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
    /// The [`Assertion`](crate::Assertion) (`balance`) account directive
    Assertion(Assertion<'a>),
    /// The [`Include`](crate::Include) directive
    Include(Include),
    /// The [`Pad`](crate::Pad) directive
    Pad(Pad<'a>),
    /// The [`Commodity`](crate::Commodity) directive
    #[cfg(feature = "unstable")]
    Commodity(Commodity<'a>),
    /// The [`Option`](crate::Option) directive
    #[cfg(feature = "unstable")]
    Option(crate::Option<'a>),
    /// The [`Event`](crate::Event) directive
    #[cfg(feature = "unstable")]
    Event(Event<'a>),
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
        match self {
            Directive::Transaction(t) => Some(t.date()),
            Directive::Open(o) => Some(o.date()),
            Directive::Close(c) => Some(c.date()),
            Directive::Price(p) => Some(p.date()),
            Directive::Assertion(a) => Some(a.date()),
            Directive::Pad(p) => Some(p.date()),
            Directive::Include(_) => None,
            #[cfg(feature = "unstable")]
            Directive::Commodity(_) | Directive::Option(_) | Directive::Event(_) => None,
        }
    }

    #[cfg(feature = "unstable")]
    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        match pair.as_rule() {
            Rule::transaction => Directive::Transaction(Transaction::from_pair(pair)),
            Rule::balance_assertion => Directive::Assertion(Assertion::from_pair(pair)),
            Rule::open_directive => Directive::Open(Open::from_pair(pair)),
            Rule::close_directive => Directive::Close(Close::from_pair(pair)),
            Rule::price_directive => Directive::Price(Price::from_pair(pair)),
            Rule::commodity => Directive::Commodity(Commodity::from_pair(pair)),
            Rule::event => Directive::Event(Event::from_pair(pair)),
            Rule::option => Directive::Option(crate::Option::from_pair(pair)),
            rule => panic!("unexpected directive rule {rule:?}"),
        }
    }
}

pub(crate) fn directive(input: Span<'_>) -> IResult<'_, Directive<'_>> {
    alt((
        map(transaction, Directive::Transaction),
        map(price, Directive::Price),
        map(open, Directive::Open),
        map(close, Directive::Close),
        map(assertion, Directive::Assertion),
        map(pad, Directive::Pad),
        map(include, Directive::Include),
    ))(input)
}

#[cfg(test)]
mod tests {
    use nom::combinator::all_consuming;

    use crate::account;

    use super::*;

    #[test]
    fn transaction() {
        let input = r#"2022-09-10 txn "My transaction""#;
        let transaction = directive(Span::new(input))
            .unwrap()
            .1
            .into_transaction()
            .unwrap();
        assert_eq!(transaction.narration(), Some("My transaction"));
    }

    #[test]
    fn price() {
        let result = directive(Span::new("2014-07-09 price CHF  5 PLN"));
        let Ok((_, Directive::Price(price))) = result else {
            panic!("Expected a price directive but was: {result:?}")
        };
        assert_eq!(price.commodity(), "CHF");
        assert_eq!(price.price().currency(), "PLN");
    }

    #[test]
    fn simple_open_directive() {
        let result = directive(Span::new(
            "2014-05-01 open Liabilities:CreditCard:CapitalOne",
        ));
        let Ok((_, Directive::Open(directive))) = result else {
          panic!("Expected an open directive but was: {result:?}")
        };
        assert_eq!(directive.account().type_(), account::Type::Liabilities);
    }

    #[test]
    fn include_directive() {
        let (_, directive) =
            all_consuming(directive)(Span::new(r#"include "myfile.beancount""#)).unwrap();
        assert_eq!(directive.date(), None);
        let Directive::Include(include) = directive else {
            panic!("Expected an include directive but was: {directive:?}")
        };
        assert_eq!(include.path().to_str(), Some("myfile.beancount"));
    }

    #[test]
    fn pad_directive() {
        let (_, directive) = all_consuming(directive)(Span::new(
            "2022-02-11 pad Assets:Cash Equity:OpeningBalances",
        ))
        .unwrap();
        assert_eq!(directive.date(), Some(Date::new(2022, 2, 11)));
        let Directive::Pad(pad) = directive else {
            panic!("Expected an include directive but was: {directive:?}")
        };
        assert_eq!(pad.target_account().components(), ["Cash"]);
        assert_eq!(pad.source_account().components(), ["OpeningBalances"]);
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
        assert!(matches!(
            directive(Span::new(input)),
            Err(nom::Err::Failure(_))
        ));
    }

    #[rstest]
    fn not_matching(#[values(" ")] input: &str) {
        assert!(matches!(
            directive(Span::new(input)),
            Err(nom::Err::Error(_))
        ));
    }

    #[rstest]
    #[case("2022-11-06 txn", Date::new(2022, 11, 6))]
    #[case("2021-02-26 open Liabilities:Debt", Date::new(2021, 2, 26))]
    #[case("2021-02-26 close Liabilities:Debt", Date::new(2021, 2, 26))]
    #[case("2014-07-09 price HOOL  600 USD", Date::new(2014, 7, 9))]
    fn date(#[case] input: &str, #[case] expected_date: Date) {
        let (_, directive) =
            directive(Span::new(input)).expect("should successfully parse directive");
        let date = directive.date().expect("directive should have a date");
        assert_eq!(date, expected_date);
    }
}
