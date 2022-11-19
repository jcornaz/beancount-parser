#![cfg(feature = "unstable")]

use beancount_parser::Parser;
use rstest::rstest;

#[rstest]
#[case("2022-11-18 txn oops", 1)]
#[case("\n\n2022-11-18 txn oops", 3)]
#[case("\n2022-11-17 txn\n2022-11-18 txn oops", 3)]
fn simple_error_line_number(#[case] input: &str, #[case] expected_line: u64) {
    let error = Parser::new(input).find_map(Result::err).unwrap();
    assert_eq!(error.line_number(), expected_line);
}

#[test]
fn error_line_number_after_multiline_transaction() {
    let input = r#"
2022-11-19 *
    Assets:Cash -10 CHF
    Expenses:Groceries

2022-11-19 txn oops"#
        .trim_start();
    let error = Parser::new(input).find_map(Result::err).unwrap();
    debug_assert_eq!(error.line_number(), 5);
}
