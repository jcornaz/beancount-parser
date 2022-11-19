#![cfg(feature = "unstable")]

use beancount_parser::Parser;
use rstest::rstest;

#[rstest]
#[case("2022-11-18 txn oops", 1)]
#[case("\n\n2022-11-18 txn oops", 3)]
#[case("\n2022-11-17 txn\n2022-11-18 txn oops", 3)]
fn simple_error(#[case] input: &str, #[case] expected_line: u64) {
    let error = Parser::new(input).find_map(Result::err).unwrap();
    assert_eq!(error.line_number(), expected_line);
}
