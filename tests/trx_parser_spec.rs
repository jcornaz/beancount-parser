use std::collections::HashSet;

use beancount_parser_2::{
    parse, Directive, DirectiveContent, Flag, Posting, PostingPrice, Transaction,
};
use rstest::rstest;

const COMMENTS: &str = include_str!("samples/comments.beancount");
const SIMPLE: &str = include_str!("samples/simple.beancount");
const OFFICIAL: &str = include_str!("samples/official.beancount");

#[rstest]
#[case("", 0)]
#[case(COMMENTS, 0)]
#[case(SIMPLE, 3)]
#[case(OFFICIAL, 1096)]
fn should_find_all_transactions(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count = parse::<f64>(input)
        .expect("parsing should succeed")
        .directives
        .into_iter()
        .filter(|d| matches!(d.content, DirectiveContent::Transaction(_)))
        .count();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
#[case("", 0)]
#[case(COMMENTS, 0)]
#[case(SIMPLE, 12)]
#[case(OFFICIAL, 3385)]
fn should_find_all_postings(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count: usize = parse::<f64>(input)
        .expect("parsing should succeed")
        .directives
        .into_iter()
        .map(|d| match d.content {
            DirectiveContent::Transaction(trx) => trx.postings.len(),
            _ => 0,
        })
        .sum();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
#[case("2023-05-15 txn", None)]
#[case("2023-05-15 txn \"Hello world!\"", Some("Hello world!"))]
#[case("2023-05-15 txn \"payee\" \"narration\"", Some("narration"))]
#[case(
    "2023-05-15 txn \"Hello world!\" ; And a comment",
    Some("Hello world!")
)]
fn should_parse_transaction_description(#[case] input: &str, #[case] expected: Option<&str>) {
    let DirectiveContent::Transaction(trx) = parse_single_directive(input).content else {
        panic!("was not a transaction");
    };
    assert_eq!(trx.narration, expected)
}

#[rstest]
#[case("2023-05-15 txn", None)]
#[case("2023-05-15 txn \"Hello world!\"", None)]
#[case("2023-05-15 txn \"payee\" \"narration\"", Some("payee"))]
#[case(
    "2023-05-15 txn \"Hello world!\" \"\"; And a comment",
    Some("Hello world!")
)]
fn should_parse_transaction_payee(#[case] input: &str, #[case] expected: Option<&str>) {
    let DirectiveContent::Transaction(trx) = parse_single_directive(input).content else {
        panic!("was not a transaction");
    };
    assert_eq!(trx.payee, expected)
}

#[rstest]
#[case("2023-05-15 txn \"Hello world!\"", &[])]
#[case("2023-05-15 txn \"Hello world!\" ^a", &["a"])]
#[case("2023-05-15 txn \"Hello world!\" #a", &[])]
#[case("2023-05-15 txn \"Hello world!\" ^link-a ^link-b", &["link-a", "link-b"])]
#[case("2023-05-15 txn \"Hello world!\" ^link-a #tag ^link-b", &["link-a", "link-b"])]
fn should_parse_transaction_links(#[case] input: &str, #[case] expected: &[&str]) {
    let DirectiveContent::Transaction(trx) = parse_single_directive(input).content else {
        panic!("was not a transaction");
    };
    assert_eq!(
        trx.links,
        expected.iter().cloned().collect::<HashSet<&str>>()
    )
}

#[rstest]
#[case("2023-05-15 txn \"Hello world!\"", &[])]
#[case("2023-05-15 txn \"Hello world!\" ^a", &[])]
#[case("2023-05-15 txn \"Hello world!\" #a", &["a"])]
#[case("2023-05-15 txn \"Hello world!\" #tag-a #tag-b", &["tag-a", "tag-b"])]
#[case("2023-05-15 txn \"Hello world!\" #tag-a ^link #tag-b", &["tag-a", "tag-b"])]
fn should_parse_transaction_tags(#[case] input: &str, #[case] expected: &[&str]) {
    let DirectiveContent::Transaction(trx) = parse_single_directive(input).content else {
        panic!("was not a transaction");
    };
    assert_eq!(
        trx.tags,
        expected.iter().cloned().collect::<HashSet<&str>>()
    )
}

#[rstest]
#[case("2023-05-15 txn", None)]
#[case("2023-05-15 txn \"hello\"", None)]
#[case("2023-05-15 *", Some(Flag::Completed))]
#[case("2023-05-15 * \"hello\"", Some(Flag::Completed))]
#[case("2023-05-15 !", Some(Flag::Incomplete))]
#[case("2023-05-15 ! \"hello\"", Some(Flag::Incomplete))]
fn should_parse_transaction_flag(#[case] input: &str, #[case] expected: Option<Flag>) {
    let DirectiveContent::Transaction(trx) = parse_single_directive(input).content else {
        panic!("was not a transaction");
    };
    assert_eq!(trx.flag, expected)
}

#[rstest]
#[case("2014-04-23 * \"Flight to Berlin\"\n  Expenses:Flights -1230.27 USD\n  Liabilities:CreditCard", &[])]
#[case("2014-04-23 * \"Flight to Berlin\" #berlin-trip-2014\n  Expenses:Flights -1230.27 USD\n  Liabilities:CreditCard", &["berlin-trip-2014"])]
#[case("2014-04-23 * #hello-world #2023_05", &["hello-world", "2023_05"])]
fn should_parse_tags(#[case] input: &str, #[case] expected: &[&str]) {
    let expected: HashSet<_> = expected.iter().copied().collect();
    let trx = parse_single_transaction(input);
    assert_eq!(trx.tags, expected);
}

#[rstest]
#[case("2023-05-15 txn", &[])]
#[case("2023-05-15 txn\n  Assets:Cash", &["Assets:Cash"])]
#[case("2023-05-15 * \"Hello\" ; with comment \n  Assets:Cash", &["Assets:Cash"])]
#[case("2023-05-15 txn\n  Assets:Cash\n Income:Salary", &["Assets:Cash", "Income:Salary"])]
fn should_parse_posting_accounts(#[case] input: &str, #[case] expected: &[&str]) {
    let DirectiveContent::Transaction(trx) = parse_single_directive(input).content else {
        panic!("was not a transaction");
    };
    let posting_accounts: Vec<&str> = trx
        .postings
        .into_iter()
        .map(|p| p.account.as_str())
        .collect();
    assert_eq!(&posting_accounts, expected);
}

#[rstest]
#[case("2023-05-15 txn", &[])]
#[case("2023-05-15 txn\n  Assets:Cash", &[None])]
#[case("2023-05-15 * \"Hello\" ; with comment \n  Assets:Cash", &[None])]
#[case("2023-05-15 txn\n  * Assets:Cash\n  ! Income:Salary\n  Equity:Openings", &[Some(Flag::Completed), Some(Flag::Incomplete), None])]
fn should_parse_posting_flags(#[case] input: &str, #[case] expected: &[Option<Flag>]) {
    let DirectiveContent::Transaction(trx) = parse_single_directive(input).content else {
        panic!("was not a transaction");
    };
    let posting_accounts: Vec<Option<Flag>> = trx.postings.into_iter().map(|p| p.flag).collect();
    assert_eq!(&posting_accounts, expected);
}

#[rstest]
fn amount_should_be_empty_if_absent() {
    let posting = parse_single_posting("2023-05-17 *\n  Assets:Cash");
    assert!(posting.amount.is_none(), "{:?}", posting.amount);
}

#[rstest]
fn price_should_be_empty_if_absent(
    #[values("2023-05-17 *\n  Assets:Cash", "2023-05-17 *\n  Assets:Cash 10 CHF")] input: &str,
) {
    let posting = parse_single_posting(input);
    assert!(posting.price.is_none(), "{:?}", posting.price);
}

#[rstest]
#[case("10 CHF", 10.0, "CHF")]
#[case("0 USD", 0.0, "USD")]
#[case("-1 EUR", -1.0, "EUR")]
#[case("1.2 PLN", 1.2, "PLN")]
#[case(".1 PLN", 0.1, "PLN")]
#[case("1. CHF", 1.0, "CHF")]
#[case("1 + 1 CHF", 1.0 + 1.0, "CHF")]
#[case("1 + 1 + 2 CHF", 1.0 + 1.0 + 2.0, "CHF")]
#[case("1+1 CHF", 1.0 + 1.0, "CHF")]
#[case("2 - 1 CHF", 2.0 - 1.0, "CHF")]
#[case("2 + 10 - 5 CHF", 2.0 + 10.0 - 5.0, "CHF")]
#[case("2+10-5 CHF", 2.0 + 10.0 - 5.0, "CHF")]
#[case("-2+10-5 CHF", -2.0 + 10.0 - 5.0, "CHF")]
#[case("10--2 CHF", 10.0 - -2.0, "CHF")]
#[case("2 + 10 + -5 CHF", 2.0 + 10.0 + -5.0, "CHF")]
#[case("2 * 3 CHF", 2.0 * 3.0, "CHF")]
#[case("2 * 3 + 4 CHF", 2.0 * 3.0 + 4.0, "CHF")]
#[case("2*3+4 CHF", 2.0 * 3.0 + 4.0, "CHF")]
#[case("2 + 3 * 4 CHF", 2.0 + 3.0 * 4.0, "CHF")]
#[case("(2 + 3) * 4 CHF", (2.0 + 3.0) * 4.0, "CHF")]
#[case("( 2 + 3 ) * 4 CHF", (2.0 + 3.0) * 4.0, "CHF")]
#[case("2 + (3 * 4) CHF", 2.0 + (3.0 * 4.0), "CHF")]
#[case("2 * 3 * 4 CHF", 2.0 * 3.0 * 4.0, "CHF")]
#[case("2*3*4 CHF", 2.0 * 3.0 * 4.0, "CHF")]
#[case("2 / 4 CHF", 2.0 / 4.0, "CHF")]
#[case("6 / 3 / 2 CHF", 6.0 / 3.0 / 2.0, "CHF")]
#[case("6/3/2 CHF", 6.0 / 3.0 / 2.0, "CHF")]
#[case("6 * 3 / 2 CHF", 6.0 * 3.0 / 2.0, "CHF")]
#[case("6 / 3 * 2 CHF", 6.0 / 3.0 * 2.0, "CHF")]
#[case("6 / 3 + 2 CHF", 6.0 / 3.0 + 2.0, "CHF")]
#[case("6 + 3 / 2 CHF", 6.0 + 3.0 / 2.0, "CHF")]
fn should_parse_amount(
    #[case] input: &str,
    #[case] expected_value: f64,
    #[case] expected_currency: &str,
) {
    let input = format!("2023-05-17 *\n  Assets:Cash {input}");
    let amount = parse_single_posting(&input).amount.unwrap();
    assert_eq!(amount.value, expected_value);
    assert_eq!(amount.currency.as_str(), expected_currency);
}

#[rstest]
#[case("10 CHF", 10, "CHF")]
#[case("0 USD", 0, "USD")]
#[case("-1 EUR", -1, "EUR")]
#[case("1.2 PLN", 1.2, "PLN")]
#[case(".1 PLN", 0.1, "PLN")]
#[case("1. CHF", 1, "CHF")]
fn should_parse_unit_price(
    #[case] input: &str,
    #[case] expected_value: impl Into<f64>,
    #[case] expected_currency: &str,
) {
    let input = format!("2023-05-17 *\n  Assets:Cash 1 DKK @ {input}");
    let PostingPrice::Unit(amount) = parse_single_posting(&input).price.unwrap() else {
        panic!("was not unit price");
    };
    assert_eq!(amount.value, expected_value.into());
    assert_eq!(amount.currency.as_str(), expected_currency);
}

#[rstest]
#[case("10 CHF", 10, "CHF")]
#[case("0 USD", 0, "USD")]
#[case("-1 EUR", -1, "EUR")]
#[case("1.2 PLN", 1.2, "PLN")]
#[case(".1 PLN", 0.1, "PLN")]
#[case("1. CHF", 1, "CHF")]
fn should_parse_total_price(
    #[case] input: &str,
    #[case] expected_value: impl Into<f64>,
    #[case] expected_currency: &str,
) {
    let input = format!("2023-05-17 *\n  Assets:Cash 1 DKK @@ {input}");
    let PostingPrice::Total(amount) = parse_single_posting(&input).price.unwrap() else {
        panic!("was not unit price");
    };
    assert_eq!(amount.value, expected_value.into());
    assert_eq!(amount.currency.as_str(), expected_currency);
}

#[rstest]
fn cost_amount_should_be_empty_if_absent() {
    let input = "2023-05-19 *\n  Assets:Cash 10 CHF {}";
    let posting = parse_single_posting(input);
    let cost = posting.cost.unwrap().amount;
    assert!(cost.is_none(), "{cost:?}");
}

#[rstest]
fn cost_should_be_empty_if_absent(
    #[values(
        "2023-05-17 *\n  Assets:Cash",
        "2023-05-17 *\n  Assets:Cash 10 CHF",
        "2023-05-17 *\n  Assets:Cash 10 CHF @ 1 EUR"
    )]
    input: &str,
) {
    let posting = parse_single_posting(input);
    assert!(posting.cost.is_none(), "{:?}", posting.cost);
}

#[rstest]
#[case("Assets:Cash 1 CHF {1 EUR}", 1, "EUR")]
#[case("Assets:Cash 1 CHF { 1 EUR }", 1, "EUR")]
#[case("Assets:Cash 1 CHF {1 EUR} @ 4 PLN", 1, "EUR")]
fn should_parse_cost(
    #[case] input: &str,
    #[case] expected_value: impl Into<f64>,
    #[case] expected_currency: &str,
) {
    let input = format!("2023-05-17 *\n  {input}",);
    let amount = parse_single_posting(&input).cost.unwrap().amount.unwrap();
    assert_eq!(amount.value, expected_value.into());
    assert_eq!(amount.currency.as_str(), expected_currency);
}

#[rstest]
#[case("Assets:Cash 1 CHF {2023-05-19}", 2023, 5, 19)]
#[case("Assets:Cash 1 CHF {1 EUR, 2023-05-19}", 2023, 5, 19)]
#[case("Assets:Cash 1 CHF {1 EUR ,2023-05-19}", 2023, 5, 19)]
#[case("Assets:Cash 1 CHF {2023-05-19, 1 EUR}", 2023, 5, 19)]
fn should_parse_cost_date(
    #[case] input: &str,
    #[case] expected_year: u16,
    #[case] expected_month: u8,
    #[case] expected_day: u8,
) {
    let input = format!("2023-05-17 *\n  {input}",);
    let date = parse_single_posting(&input).cost.unwrap().date.unwrap();
    assert_eq!(date.year, expected_year);
    assert_eq!(date.month, expected_month);
    assert_eq!(date.day, expected_day);
}

#[rstest]
fn should_include_tag_stack() {
    let input = r#"
pushtag #foo
pushtag #bar
2022-05-27 * #baz
poptag #foo
2022-05-28 *"#;
    let beancount = parse::<f64>(input).unwrap();
    let transactions: Vec<_> = beancount
        .directives
        .into_iter()
        .map(|d| match d.content {
            DirectiveContent::Transaction(trx) => trx,
            _ => panic!("was not a transaction: {d:?}"),
        })
        .collect();
    assert_eq!(
        transactions.len(),
        2,
        "unexpected number of transactions: {transactions:?}"
    );
    assert_eq!(
        transactions[0].tags,
        ["foo", "bar", "baz"].into_iter().collect::<HashSet<_>>()
    );
    assert_eq!(
        transactions[1].tags,
        ["bar"].into_iter().collect::<HashSet<_>>()
    );
}

#[rstest]
fn should_reject_invalid_input(
    #[values(
        "2023-05-15txn \"narration\"",
        "2023-05-15* \"narration\"",
        "2023-05-15! \"narration\"",
        "2023-05-15 txn\"narration\"",
        "2023-05-15txn \"payee\" \"narration\"",
        "2023-05-15 txn\"payee\" \"narration\"",
        "2023-05-15 txn \"payee\"\"narration\"",
        "2023-05-15 * \"payee\"\"narration\"",
        "2023-05-15 txn\nAssets:Cash",
        "2023-05-15 * \"hello\"\nAssets:Cash",
        "2023-05-15 * \"test\"\n  *Assets:Cash",
        "2023-05-15 * \"test\"\n* Assets:Cash",
        "2023-05-15 * \"test\"\n  Assets:Cash10 CHF",
        "2023-05-15 * \"test\"\n  Assets:Cash 10CHF",
        "2023-05-15 * \"test\"\n  Assets:Cash 10..2 CHF",
        "2023-05-15 * \"test\"\n  Assets:Cash - CHF",
        "2023-05-19 *\n  Assets:Cash 1 CHF @2 EUR",
        "2023-05-19 *\n  Assets:Cash 1 CHF@ 2 EUR",
        "2023-05-19 *\n  Assets:Cash @ 2 EUR",
        "2023-05-19 *\n  Assets:Cash {1 EUR} @ 4 PLN",
        "2023-05-19 *\n  Assets:Cash {1 EUR}",
        "2023-05-19 *\n  Assets:Cash 1 CHF {1 EUR}@ 4 PLN",
        "2023-05-19 *\n  Assets:Cash 1 CHF {1 EUR} @4 PLN",
        "2023-05-19 *\n  Assets:Cash 1 CHF {1 EUR,}",
        "2023-05-19 *\n  Assets:Cash 1 CHF {, 2023-05-19}",
        "2023-05-19 *\n  Assets:Cash 1 CHF {,}",
        "2014-04-23 * #hello-world#2023_05",
        "2014-04-23 *#hello-world #2023_05",
        "pushtag#test",
        "pushtag test",
        "pushtag",
        "poptagtest",
        "poptag#test",
        "poptag test",
        "poptag",
        "poptagtest"
    )]
    input: &str,
) {
    let result = parse::<f64>(input);
    assert!(result.is_err(), "{result:#?}");
}

fn parse_single_directive(input: &str) -> Directive<f64> {
    let directives = parse(input).expect("parsing should succeed").directives;
    assert_eq!(
        directives.len(),
        1,
        "unexepcted number of directives: {:?}",
        directives
    );
    directives.into_iter().next().unwrap()
}

fn parse_single_posting(input: &str) -> Posting<'_, f64> {
    let trx = parse_single_transaction(input);
    assert_eq!(
        trx.postings.len(),
        1,
        "unexpected number of postings: {:?}",
        trx.postings
    );
    trx.postings.into_iter().next().unwrap()
}

fn parse_single_transaction(input: &str) -> Transaction<f64> {
    let directive_content = parse_single_directive(input).content;
    let DirectiveContent::Transaction(trx) = directive_content else {
        panic!("was not a transaction but: {directive_content:?}");
    };
    trx
}
