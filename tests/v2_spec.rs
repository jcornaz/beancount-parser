#![cfg(feature = "unstable")]

const COMMENTS: &str = include_str!("../tests/samples/comments.beancount");
// TODO const SIMPLE: &str = include_str!("../tests/samples/simple.beancount");
// TODO const OFFICIAL: &str = include_str!("../tests/samples/official.beancount");

use beancount_parser::v2::parse;
use rstest::rstest;

#[rstest]
fn should_succeed_for_valid_input(#[values("", COMMENTS)] input: &str) {
    parse(input).expect("parsing should succeed");
}
