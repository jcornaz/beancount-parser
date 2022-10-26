mod utils;

use beancount_parser::{Directive, Parser};
use rstest::rstest;

use crate::utils::DirectiveList;

const SIMPLE: &str = include_str!("examples/simple.beancount");
const COMMENTS: &str = include_str!("examples/comments.beancount");

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
fn pushtags_adds_tag_to_next_transaction() {
    let input = "pushtag #hello\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert_eq!(transaction.tags(), &["hello"]);
}

#[test]
fn multiple_pushtags_add_tags_to_next_transaction() {
    let input = "pushtag #hello\npushtag #world\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert_eq!(transaction.tags(), &["hello", "world"]);
}

#[test]
fn poptag_removes_tag_from_stack() {
    let input = "pushtag #hello\npoptag #hello\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert!(transaction.tags().is_empty());
}

#[test]
fn poptag_removes_only_concerned_tag_from_stack() {
    let input = "pushtag #hello\npushtag #world\npoptag #hello\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert_eq!(transaction.tags(), &["world"]);
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
