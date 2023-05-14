#![cfg(feature = "unstable")]

const COMMENTS: &str = include_str!("samples/comments.beancount");
// TODO const SIMPLE: &str = include_str!("samples/simple.beancount");
// TODO const OFFICIAL: &str = include_str!("samples/official.beancount");

use beancount_parser::v2::{parse, Directive};
use rstest::rstest;

#[rstest]
fn should_succeed_for_valid_input(#[values("", "\n", COMMENTS)] input: &str) {
    parse(input).expect("parsing should succeed");
}

#[rstest]
#[case("2014-05-01 open Assets:Cash", 2014, 5, 1)]
#[case("0001-01-01 open Assets:Cash", 1, 1, 1)]
fn should_parse_date(
    #[case] input: &str,
    #[case] expected_year: u16,
    #[case] expected_month: u8,
    #[case] expected_day: u8,
) {
    let date = parse_single_directive(input).date();
    assert_eq!(date.year, expected_year);
    assert_eq!(date.month_of_year, expected_month);
    assert_eq!(date.day_of_month, expected_day);
}

fn parse_single_directive(input: &str) -> Directive {
    let directives = parse(input).expect("parsing should succeed").directives;
    assert_eq!(
        directives.len(),
        1,
        "unexepcted number of directives: {}",
        directives.len()
    );
    directives.into_iter().next().unwrap()
}

#[rstest]
fn should_reject_invalid_input(
    #[values(
        "14-05-01 open Assets:Cash",
        "2014-5-01 open Assets:Cash",
        "2014-05-1 open Assets:Cash",
        "2014-00-01 open Assets:Cash",
        "2014-13-01 open Assets:Cash",
        "2014-05-00 open Assets:Cash",
        "2014-05-32 open Assets:Cash"
    )]
    input: &str,
) {
    let result = parse(input);
    assert!(result.is_err(), "{result:#?}");
}
