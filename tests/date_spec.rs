mod utils;

use beancount_parser::{Date, Parser};
use rstest::rstest;
use utils::DirectiveList;

fn parse_valid_date(input: &str) -> Date {
    Parser::new(&format!("{input} txn"))
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction")
        .date()
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
