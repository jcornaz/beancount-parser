use beancount_parser::{Directive, Parser};
use rstest::rstest;

const EXAMPLE1: &str = include_str!("examples/example1.beancount");
const COMMENTS: &str = include_str!("examples/comments.beancount");

#[rstest]
fn valid_examples_should_not_return_an_error(
    #[values("", " \n ", EXAMPLE1, COMMENTS)] input: &str,
) {
    for result in Parser::new(input) {
        assert!(result.is_ok());
    }
}

#[rstest]
#[case("", 0)]
#[case(EXAMPLE1, 3)]
#[case(COMMENTS, 0)]
fn examples_have_expected_number_of_transaction(
    #[case] input: &str,
    #[case] expected_count: usize,
) {
    let actual_count = Parser::new(input)
        .filter(|d| matches!(d, Ok((_, Directive::Transaction(_)))))
        .count();
    assert_eq!(actual_count, expected_count);
}
