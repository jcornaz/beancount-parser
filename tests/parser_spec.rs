const COMMENTS: &str = include_str!("samples/comments.beancount");
const SIMPLE: &str = include_str!("samples/simple.beancount");
// TODO const OFFICIAL: &str = include_str!("samples/official.beancount");

use beancount_parser::{metadata, parse, Decimal, Directive, DirectiveContent, Flag, Posting};
use rstest::rstest;

#[rstest]
fn should_succeed_for_valid_input(#[values("", "\n", COMMENTS, SIMPLE)] input: &str) {
    parse(input).expect("parsing should succeed");
}

#[rstest]
#[case("", 0)]
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
#[case("", 0)]
#[case(SIMPLE, 10)]
fn should_find_all_open_directives(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count = parse(input)
        .expect("parsing should succeed")
        .directives
        .into_iter()
        .filter(|d| matches!(d.content, DirectiveContent::Open(_)))
        .count();
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
#[case("2014-05-01 open Assets:Cash", 2014, 5, 1)]
#[case("0001-01-01 open Assets:Cash CHF", 1, 1, 1)]
#[case("0003-02-01 close Assets:Cash", 3, 2, 1)]
#[case("0001-02-03 txn", 1, 2, 3)]
#[case("0001-02-03 * \"hello\"", 1, 2, 3)]
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
#[case(
    "2020-04-10 balance Assets:US:BofA:Checking        2473.33 USD",
    "Assets:US:BofA:Checking"
)]
fn should_parse_balance_assertion_account(#[case] input: &str, #[case] exepected: &str) {
    let DirectiveContent::Balance(assertion) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(assertion.account.as_str(), exepected);
}

#[rstest]
#[case(
    "2020-04-10 balance Assets:US:BofA:Checking        2473 USD",
    2473,
    "USD"
)]
fn should_parse_balance_assertion_amount(
    #[case] input: &str,
    #[case] expected_value: impl Into<Decimal>,
    #[case] expected_currency: &str,
) {
    let DirectiveContent::Balance(assertion) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(assertion.amount.value, expected_value.into());
    assert_eq!(assertion.amount.currency.as_str(), expected_currency);
}

#[rstest]
#[case("2014-05-01 open Assets:Cash", "Assets:Cash")]
#[case("2014-05-01 open Liabilities:A", "Liabilities:A")]
#[case("2014-05-01 open Equity:A", "Equity:A")]
#[case("2014-05-01 open Income:A", "Income:A")]
#[case("2014-05-01 open Expenses:A", "Expenses:A")]
#[case("2014-05-01 open Assets:Cash2", "Assets:Cash2")]
#[case("2014-05-01 open Assets:Hello-world", "Assets:Hello-world")]
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
#[case("2014-05-01 open Assets:Checking", &[])]
#[case("2014-05-01 open Assets:Checking USD", &["USD"])]
#[case("2014-05-01 open Assets:Checking A", &["A"])]
#[case("2014-05-01 open Assets:Checking USD'42-CHF_EUR.PLN", &["USD'42-CHF_EUR.PLN"])]
#[case("2014-05-01 open Assets:Checking\tUSD", &["USD"])]
#[case("2014-05-01 open Assets:Checking  USD", &["USD"])]
#[case("2014-05-01 open Assets:Checking CHF,USD", &["CHF", "USD"])]
#[case("2014-05-01 open Assets:Checking CHF, USD", &["CHF", "USD"])]
#[case("2014-05-01 open Assets:Checking CHF  ,\tUSD", &["CHF", "USD"])]
fn should_parse_open_account_currency(#[case] input: &str, #[case] exepcted_currencies: &[&str]) {
    let DirectiveContent::Open(open) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    let actual_currencies: Vec<&str> = open.currencies.iter().map(|c| c.as_str()).collect();
    assert_eq!(&actual_currencies, exepcted_currencies);
}

#[rstest]
#[case("2014-05-01 close Assets:Cash", "Assets:Cash")]
#[case("2014-05-01  close  Assets:Cash", "Assets:Cash")]
#[case("2014-05-01\tclose\tAssets:Cash:Wallet", "Assets:Cash:Wallet")]
#[case("2014-05-01\tclose\tAssets:Cash:Wallet  ", "Assets:Cash:Wallet")]
#[case(
    "2014-05-01\tclose\tAssets:Cash:Wallet ; And a comment",
    "Assets:Cash:Wallet"
)]
fn should_parse_close_account(#[case] input: &str, #[case] expected_account: &str) {
    let DirectiveContent::Close(close) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(close.account.as_str(), expected_account);
}

#[rstest]
fn should_parse_option() {
    let options = parse(r#"option "Hello" "world!""#).unwrap().options;
    assert_eq!(options.get("Hello"), Some(&"world!"));
}

#[rstest]
fn should_parse_commodity() {
    let input = "1792-01-01 commodity USD";
    let DirectiveContent::Commodity(commodity) = parse_single_directive(input).content else {
        panic!("was not an commodity directive");
    };
    assert_eq!(commodity.as_str(), "USD");
}

#[rstest]
fn should_parse_event() {
    let input = "2020-12-09 event \"location\" \"New Metropolis\"";
    let DirectiveContent::Event(event) = parse_single_directive(input).content else {
        panic!("was not an commodity directive");
    };
    assert_eq!(event.name, "location");
    assert_eq!(event.value, "New Metropolis");
}

#[rstest]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"",
    "title",
    metadata::Value::String("hello")
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n  name: \"world\"",
    "title",
    metadata::Value::String("hello")
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n  name: \"world\"",
    "name",
    metadata::Value::String("world")
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"",
    "title",
    metadata::Value::String("hello")
)]
#[case(
    "2022-05-18 *\n  goodTitle: \"Hello world!\"",
    "goodTitle",
    metadata::Value::String("Hello world!")
)]
#[case(
    "2022-05-18 *\n  good-title: \"Hello world!\"",
    "good-title",
    metadata::Value::String("Hello world!")
)]
#[case(
    "2022-05-18 *\n  good_title: \"Hello world!\"",
    "good_title",
    metadata::Value::String("Hello world!")
)]
#[case(
    "2022-05-18 *\n  good_title2: \"Hello world!\"",
    "good_title2",
    metadata::Value::String("Hello world!")
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"\n  Assets:Cash 10 CHF",
    "title",
    metadata::Value::String("hello")
)]
fn should_parse_metadata_entry(
    #[case] input: &str,
    #[case] key: &str,
    #[case] expected_value: metadata::Value<'static>,
) {
    let metdata = parse_single_directive(input).metadata;
    assert_eq!(metdata.get(key), Some(&expected_value));
}

#[rstest]
fn should_reject_invalid_input(
    #[values(
        "14-05-01 open Assets:Cash",
        "2014-05-05 open Assets",
        "2014-05-05 open Assets:hello",
        "2014-05-05 open Assets:2",
        "2014-05-05 open Assets:2Hello",
        "2014-5-01 open Assets:Cash",
        "2014-05-1 open Assets:Cash",
        "2014-00-01 open Assets:Cash",
        "2014-13-01 open Assets:Cash",
        "2014-05-00 open Assets:Cash",
        "2014-05-32 open Assets:Cash",
        "2014-05-15 open Assets::Cash",
        "2014-05-01 open Assets:Cash 2014-05-01 open Assets:Cash",
        "2014-05-01open Assets:Cash",
        "2014-05-01 openAssets:Cash",
        "2014-05-01 open",
        "2014-05-01 close",
        "2014-05-01 open oops",
        "2014-05-01 close oops",
        "2014-05-01 open Assets:Checking usd",
        "2014-05-01 open Assets:Checking Hello",
        "2014-05-01 open Assets:Checking USD CHF",
        "2014-05-01 open Assets:Checking 1SD",
        "2014-05-01 open Assets:Checking US2",
        "2014-05-01 open Assets:Checking US-",
        "2014-05-01 open Assets:Checking -US",
        "2014-05-01close Assets:Cash",
        "2014-05-01 closeAssets:Cash",
        "2014-05-01 closeAssets:Cash",
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
        "2023-05-15 commodity",
        "2023-05-15commodity CHF",
        "2023-05-15 commodityCHF",
        "option\"hello\" \"world\"",
        "option \"hello\"\"world\"",
        "option \"hello\"",
        "2022-05-18 open Assets:Cash\ntitle: \"hello\"",
        "2022-05-18 open Assets:Cash\n  Title: \"hello\"",
        "2020-04-10 balance Assets:US:BofA:Checking2473.33 USD",
        "2020-04-10 balance Assets:US:BofA:Checking",
        "2020-12-09 event \"location\"\"New Metropolis\"",
        "2020-12-09 event\"location\" \"New Metropolis\"",
        "2020-12-09 event \"location\"",
        "2020-12-09 event"
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
