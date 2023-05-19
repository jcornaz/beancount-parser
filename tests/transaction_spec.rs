use beancount_parser_2::{parse, Decimal, Directive, DirectiveContent, Flag, Posting};
use rstest::rstest;

const COMMENTS: &str = include_str!("samples/comments.beancount");
const SIMPLE: &str = include_str!("samples/simple.beancount");
// TODO const OFFICIAL: &str = include_str!("samples/official.beancount");

#[rstest]
#[case("", 0)]
#[case(COMMENTS, 0)]
#[case(SIMPLE, 3)]
fn should_find_all_transactions(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count = parse(input)
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
fn should_find_all_postings(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count: usize = parse(input)
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
#[case("10 CHF", 10, "CHF")]
#[case("0 USD", 0, "USD")]
#[case("-1 EUR", -1, "EUR")]
#[case("1.2 PLN", Decimal::new(12, 1), "PLN")]
#[case(".1 PLN", Decimal::new(1, 1), "PLN")]
#[case("1. CHF", 1, "CHF")]
fn should_parse_amount_if_set(
    #[case] input: &str,
    #[case] expected_value: impl Into<Decimal>,
    #[case] expected_currency: &str,
) {
    let input = format!("2023-05-17 *\n  Assets:Cash {input}");
    let amount = parse_single_posting(&input).amount.unwrap();
    assert_eq!(amount.value, expected_value.into());
    assert_eq!(amount.currency.as_str(), expected_currency);
}

#[rstest]
#[case("10 CHF", 10, "CHF")]
#[case("0 USD", 0, "USD")]
#[case("-1 EUR", -1, "EUR")]
#[case("1.2 PLN", Decimal::new(12, 1), "PLN")]
#[case(".1 PLN", Decimal::new(1, 1), "PLN")]
#[case("1. CHF", 1, "CHF")]
fn should_parse_price_if_set(
    #[case] input: &str,
    #[case] expected_value: impl Into<Decimal>,
    #[case] expected_currency: &str,
) {
    let input = format!("2023-05-17 *\n  Assets:Cash 1 DKK @ {input}");
    let amount = parse_single_posting(&input).price.unwrap();
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
    #[case] expected_value: impl Into<Decimal>,
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
        "2023-05-19 *\n  Assets:Cash 1 CHF {,}"
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
        "unexepcted number of directives: {:?}",
        directives
    );
    directives.into_iter().next().unwrap()
}

fn parse_single_posting(input: &str) -> Posting<'_> {
    let directive_content = parse_single_directive(input).content;
    let DirectiveContent::Transaction(trx) = directive_content else {
        panic!("was not a transaction but: {directive_content:?}");
    };
    assert_eq!(
        trx.postings.len(),
        1,
        "unexpected number of postings: {:?}",
        trx.postings
    );
    trx.postings.into_iter().next().unwrap()
}
