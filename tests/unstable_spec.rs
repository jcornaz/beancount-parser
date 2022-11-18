#![cfg(feature = "unstable")]

use beancount_parser::Parser;
use rstest::rstest;

#[rstest]
#[case("2022-11-18 txn oops", 1)]
#[case("\n\n2022-11-18 txn oops", 3)]
fn error_on_first_directive(#[case] input: &str, #[case] expected_line: u64) {
    let result = Parser::new(input).next();
    let Some(Err(error)) = result else { panic!("Expected an error but was: {result:?}") };
    assert_eq!(error.line_number(), expected_line);
}
