use rstest::rstest;
use rust_decimal::Decimal;

use beancount_parser::{parse, Directive, DirectiveContent, Posting, Transaction};

#[rstest]
#[case("10 CHF", 10, "CHF")]
#[case("0 USD", 0, "USD")]
#[case("-1 EUR", -1, "EUR")]
#[case("1.2 PLN", Decimal::new(12, 1), "PLN")]
#[case(".1 PLN", Decimal::new(1, 1), "PLN")]
#[case("1. CHF", 1, "CHF")]
fn should_parse_amount(
    #[case] input: &str,
    #[case] expected_value: impl Into<Decimal>,
    #[case] expected_currency: &str,
) {
    let input = format!("2023-05-17 *\n  Assets:Cash {input}");
    let amount = parse_single_posting(&input).amount.unwrap();
    assert_eq!(amount.value, expected_value.into());
    assert_eq!(amount.currency.as_str(), expected_currency);
}

fn parse_single_directive(input: &str) -> Directive<Decimal> {
    let directives = parse(input).expect("parsing should succeed").directives;
    assert_eq!(
        directives.len(),
        1,
        "unexpected number of directives: {:?}",
        directives
    );
    directives.into_iter().next().unwrap()
}

fn parse_single_posting(input: &str) -> Posting<Decimal> {
    let trx = parse_single_transaction(input);
    assert_eq!(
        trx.postings.len(),
        1,
        "unexpected number of postings: {:?}",
        trx.postings
    );
    trx.postings.into_iter().next().unwrap()
}

fn parse_single_transaction(input: &str) -> Transaction<Decimal> {
    let directive_content = parse_single_directive(input).content;
    let DirectiveContent::Transaction(trx) = directive_content else {
        panic!("was not a transaction but: {directive_content:?}");
    };
    trx
}
