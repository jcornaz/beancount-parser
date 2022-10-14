mod utils;

use beancount_parser::{account, Directive, Parser};
use rstest::rstest;

use crate::utils::{assert_date_eq, DirectiveList};

const SIMPLE: &str = include_str!("examples/simple.beancount");
const COMMENTS: &str = include_str!("examples/comments.beancount");
#[allow(unused)]
const OFFICIAL: &str = include_str!("examples/official.beancount");

#[rstest]
fn valid_examples_should_not_return_an_error(
    #[values("", " \n ", SIMPLE, COMMENTS, OFFICIAL)] input: &str,
) {
    let mut count = 0;
    for result in Parser::new(input) {
        count += 1;
        assert!(
            result.is_ok(),
            "The {count} directive is an error: {result:?}"
        );
    }
}

#[rstest]
#[case("", 0)]
#[case(SIMPLE, 3)]
#[case(COMMENTS, 0)]
fn examples_have_expected_number_of_transaction(
    #[case] input: &str,
    #[case] expected_count: usize,
) {
    let actual_count = Parser::new(input)
        .filter_map(Result::ok)
        .filter_map(Directive::into_transaction)
        .count();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
#[case("", 0)]
#[case(SIMPLE, 13)]
#[case(COMMENTS, 0)]
fn examples_have_expected_number_of_postings(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count: usize = Parser::new(input)
        .filter_map(Result::ok)
        .filter_map(Directive::into_transaction)
        .map(|t| t.postings().len())
        .sum();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
fn invalid_examples_return_an_error(#[values("2022-09-10 txn Oops...")] input: &str) {
    let items = Parser::new(input).collect::<Vec<Result<_, _>>>();
    assert!(items[0].is_err());
}

#[test]
fn transaction_with_lot_date() {
    let beancount = r#"
2020-10-08 * "Buy shares of VEA"
  Assets:US:ETrade:VEA                                 34 VEA {100 USD, 2020-10-08}
"#;
    let transaction = Parser::new(beancount)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    let cost = transaction.postings()[0]
        .cost()
        .expect("should have a cost");
    assert_eq!(cost.value().try_into_f64().unwrap(), 100.0);
    assert_eq!(cost.currency(), "USD");
}

#[test]
fn parse_price_directive() {
    let beancount = "2014-07-09 price CHF  5 PLN";
    let directive = match Parser::new(beancount).assert_single_directive() {
        Directive::Price(price) => price,
        d => panic!("Was not a price directive: {d:?}"),
    };
    assert_date_eq(directive.date(), 2014, 7, 9);
    assert_eq!(directive.commodity(), "CHF");
    assert_eq!(directive.price().value().try_into_f64().unwrap(), 5.0);
    assert_eq!(directive.price().currency(), "PLN");
}

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
