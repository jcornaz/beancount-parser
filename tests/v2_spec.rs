const COMMENTS: &str = include_str!("samples/comments.beancount");
// TODO const SIMPLE: &str = include_str!("samples/simple.beancount");
// TODO const OFFICIAL: &str = include_str!("samples/official.beancount");

use beancount_parser::{parse, Directive, DirectiveContent};
use rstest::rstest;

#[rstest]
fn should_succeed_for_valid_input(#[values("", "\n", COMMENTS)] input: &str) {
    parse(input).expect("parsing should succeed");
}

// TODO test number of open directives

#[rstest]
#[case("2014-05-01 open Assets:Cash", 2014, 5, 1)]
#[case("0001-01-01 open Assets:Cash", 1, 1, 1)]
fn should_parse_date(
    #[case] input: &str,
    #[case] expected_year: u16,
    #[case] expected_month: u8,
    #[case] expected_day: u8,
) {
    let date = parse_single_directive(input).date;
    assert_eq!(date.year, expected_year);
    assert_eq!(date.month_of_year, expected_month);
    assert_eq!(date.day_of_month, expected_day);
}

#[rstest]
#[case("2014-05-01 open Assets:Cash", "Assets:Cash")]
#[case("2014-05-01  open  Assets:Cash", "Assets:Cash")]
#[case("2014-05-01\topen\tAssets:Cash:Wallet", "Assets:Cash:Wallet")]
#[case("2014-05-01\topen\tAssets:Cash:Wallet  ", "Assets:Cash:Wallet")]
#[case(
    "2014-05-01\topen\tAssets:Cash:Wallet ; And a comment",
    "Assets:Cash:Wallet"
)]
fn should_parse_open_account(#[case] input: &str, #[case] expected_account: &str) {
    let DirectiveContent::Open(open) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(open.account.as_str(), expected_account);
}

#[rstest]
fn should_reject_invalid_input(
    #[values(
        "14-05-01 open Assets:Cash",
        "14-05-05 open Assets",
        "2014-5-01 open Assets:Cash",
        "2014-05-1 open Assets:Cash",
        "2014-00-01 open Assets:Cash",
        "2014-13-01 open Assets:Cash",
        "2014-05-00 open Assets:Cash",
        "2014-05-32 open Assets:Cash",
        "2014-05-15 open Assets::Cash",
        "2014-05-01 open Assets:Cash 2014-05-01 open Assets:Cash",
        // TODO no new line between directives
        // TODO "2014-05-01open Assets:Cash",
        // TODO "2014-05-01 openAssets:Cash",
        // TODO "2014-05-01 open",
        // TODO "2014-05-01 open oops"
    )]
    input: &str,
) {
    let result = parse(input);
    assert!(result.is_err(), "{result:#?}");
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
