use beancount_parser::{Directive, Parser};
use rstest::rstest;

const SIMPLE: &str = include_str!("examples/simple.beancount");
const COMMENTS: &str = include_str!("examples/comments.beancount");

#[rstest]
fn valid_examples_should_not_return_an_error(#[values("", " \n ", SIMPLE, COMMENTS)] input: &str) {
    for result in Parser::new(input) {
        assert!(result.is_ok());
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
        .filter(|d| matches!(d, Ok((_, Directive::Transaction(_)))))
        .count();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
#[case("", 0)]
#[case(SIMPLE, 13)]
#[case(COMMENTS, 0)]
fn examples_have_expected_number_of_postings(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count: usize = Parser::new(input)
        .filter_map(|d| {
            d.ok()
                .as_ref()
                .and_then(|(_, d)| d.as_transaction())
                .map(|t| t.postings().len())
        })
        .sum();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
fn error(#[values("2022-09-10 txn Oops...")] input: &str) {
    let items = Parser::new(input).collect::<Vec<Result<_, _>>>();
    assert!(items[0].is_err());
}
