use beancount_parser::Parser;
use rstest::rstest;

const EXAMPLE1: &str = include_str!("examples/example1.beancount");

#[rstest]
fn valid_examples_should_not_return_an_error(#[values(EXAMPLE1)] input: &str) {
    for result in Parser::new(input) {
        assert!(result.is_ok());
    }
}
