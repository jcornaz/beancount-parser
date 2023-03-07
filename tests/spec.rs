use rstest::rstest;

use beancount_parser::{Date, Directive, Parser};
use samples::{COMMENTS, OFFICIAL, SIMPLE};
use utils::assert_single_directive;

mod samples;
mod utils;

#[rstest]
fn valid_examples_do_not_return_an_error(
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

#[rstest]
fn comment_line(
    #[values(
        "",
        "\n",
        "2016 - 11 - 28 close Liabilities:CreditCard:CapitalOne",
        "Hello world",
        "* Banking",
        "** Bank of America",
        ";; Transactions follow …",
        "; foo bar"
    )]
    input: &str,
) {
    let directives = Parser::new(input)
        .collect::<Result<Vec<_>, _>>()
        .expect("successful parse");
    assert_eq!(directives.len(), 0);
}

#[test]
fn close_directive() {
    let directive = assert_single_directive("2016-11-28 close Liabilities:CreditCard:CapitalOne");
    let Directive::Close(directive) = directive else {
        panic!("Expected a close directive but was: {directive:?}")
    };
    assert_eq!(directive.date(), Date::new(2016, 11, 28));
    assert_eq!(
        directive.account().components(),
        &["CreditCard", "CapitalOne"]
    );
}
