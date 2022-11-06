mod utils;

use beancount_parser::{Date, Directive, Parser};
use rstest::rstest;
use utils::{assert_date_eq, DirectiveList};

fn parse_valid_date(input: &str) -> Date {
    Parser::new(&format!("{input} txn"))
        .assert_single_directive()
        .date()
        .expect("should be a transaction")
}

#[rstest]
#[case("2018-11-07", "2018-11-08")]
#[case("2018-11-08", "2018-12-07")]
#[case("2017-11-08", "2018-11-07")]
fn date_comparison(#[case] before: &str, #[case] after: &str) {
    let before = parse_valid_date(before);
    let after = parse_valid_date(after);
    assert!(before < after);
    assert!(after > before);
}

#[rstest]
#[case("2022-11-06 txn", 2022, 11, 6)]
#[case("2021-02-26 open Liabilities:Debt", 2021, 2, 26)]
#[case("2021-02-26 close Liabilities:Debt", 2021, 2, 26)]
#[case("2014-07-09 price HOOL  600 USD", 2014, 7, 9)]
fn date_on_directive(
    #[case] input: &str,
    #[case] expected_year: u16,
    #[case] expected_month: u8,
    #[case] expected_day: u8,
) {
    let directive: Directive = Parser::new(input).assert_single_directive();
    let date = directive.date().unwrap();
    assert_date_eq(date, expected_year, expected_month, expected_day);
}
