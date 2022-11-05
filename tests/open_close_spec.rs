mod utils;

use beancount_parser::{account, Directive, Parser};

use crate::utils::{assert_date_eq, DirectiveList};

#[test]
fn simple_open_directive() {
    let input = "2014-05-01 open Liabilities:CreditCard:CapitalOne";
    let directive = match Parser::new(input).assert_single_directive() {
        Directive::Open(d) => d,
        d => panic!("unexpectied directive type: {d:?}"),
    };
    assert_date_eq(directive.date(), 2014, 5, 1);
    assert_eq!(directive.account().type_(), account::Type::Liabilities);
    assert_eq!(
        directive.account().components(),
        &["CreditCard", "CapitalOne"]
    );
}

#[test]
fn open_with_single_currency_constraint() {
    let input = "2014-05-01 open Liabilities:CreditCard:CapitalOne CHF";
    let directive = match Parser::new(input).assert_single_directive() {
        Directive::Open(d) => d,
        d => panic!("unexpectied directive type: {d:?}"),
    };
    assert_eq!(directive.currencies(), &["CHF"]);
}

#[test]
fn open_with_multipl_currency_constraints() {
    let input = "2014-05-01 open Liabilities:CreditCard:CapitalOne CHF, USD,EUR";
    let directive = match Parser::new(input).assert_single_directive() {
        Directive::Open(d) => d,
        d => panic!("unexpectied directive type: {d:?}"),
    };
    assert_eq!(directive.currencies(), &["CHF", "USD", "EUR"]);
}
