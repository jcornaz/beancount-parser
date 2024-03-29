#![allow(clippy::items_after_test_module, clippy::pedantic)]

use std::{collections::HashSet, path::Path};

use rstest::rstest;

use beancount_parser::{metadata, parse, Account, BeancountFile, Directive, DirectiveContent};

const COMMENTS: &str = include_str!("samples/comments.beancount");
const SIMPLE: &str = include_str!("samples/simple.beancount");
const OFFICIAL: &str = include_str!("samples/official.beancount");

#[rstest]
fn should_succeed_for_valid_input(#[values("", "\n", COMMENTS, SIMPLE, OFFICIAL)] input: &str) {
    parse::<f64>(input).expect("parsing should succeed");
}

#[rstest]
#[case("", 0)]
#[case(SIMPLE, 12)]
fn should_find_all_open_directives(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count = parse::<f64>(input)
        .expect("parsing should succeed")
        .directives
        .into_iter()
        .filter(|d| matches!(d.content, DirectiveContent::Open(_)))
        .count();
    assert_eq!(actual_count, expected_count);
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
    assert_eq!(date.month, expected_month);
    assert_eq!(date.day, expected_day);
}

#[rstest]
#[case(
    "2020-04-10 balance Assets:US:BofA:Checking        2473.33 USD",
    "Assets:US:BofA:Checking"
)]
fn should_parse_balance_assertion_account(#[case] input: &str, #[case] expected: &str) {
    let DirectiveContent::Balance(assertion) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(assertion.account.as_str(), expected);
}

#[rstest]
#[case(
    "2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances",
    "Assets:BofA:Checking"
)]
fn should_parse_pad_account(#[case] input: &str, #[case] expected: &str) {
    let DirectiveContent::Pad(pad) = parse_single_directive(input).content else {
        panic!("was not a pad directive");
    };
    assert_eq!(pad.account.as_str(), expected);
}

#[rstest]
#[case(
    "2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances",
    "Equity:Opening-Balances"
)]
fn should_parse_pad_source_account(#[case] input: &str, #[case] expected: &str) {
    let DirectiveContent::Pad(pad) = parse_single_directive(input).content else {
        panic!("was not a pad directive");
    };
    assert_eq!(pad.source_account.as_str(), expected);
}

#[rstest]
#[case(
    "2013-09-10 balance Assets:US:Vanguard  305.205 RGAGX",
    305.205,
    None,
    "RGAGX"
)]
#[case(
    "2013-09-10 balance Assets:US:Vanguard  305.205 ~ 0.002 RGAGX",
    305.205,
    Some(0.002),
    "RGAGX"
)]
#[case(
    "2013-09-10 balance Assets:US:Vanguard  305.205~0.002 RGAGX",
    305.205,
    Some(0.002),
    "RGAGX"
)]
fn should_parse_balance_assertion_amount(
    #[case] input: &str,
    #[case] expected_value: f64,
    #[case] expected_tolerance: Option<f64>,
    #[case] expected_currency: &str,
) {
    let DirectiveContent::Balance(assertion) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(assertion.amount.value, expected_value);
    assert_eq!(assertion.amount.currency.as_str(), expected_currency);
    assert_eq!(assertion.tolerance, expected_tolerance);
}

#[rstest]
#[case::assets("Assets:A")]
#[case::liabilities("Liabilities:A")]
#[case::equity("Equity:A")]
#[case::expenses("Expenses:A")]
#[case::income("Income:A")]
#[case::one_component("Assets:Cash")]
#[case::multiple_components("Assets:Cash:Wallet")]
#[case::dash("Assets:Hello-world")]
#[case::num_at_end("Assets:Cash2")]
#[case::num_at_start("Assets:2Cash")]
#[case::non_standard_name("Ausgaben:A")]
fn account_from_str_should_parse_valid_account(#[case] input: &str) {
    let account: Account = input.parse().unwrap();
    assert_eq!(account.as_str(), input);
}

#[rstest]
#[case("oops")]
#[case("Assets:")]
#[case("Assets::Cash")]
fn account_from_str_should_fail_for_invalid_input(#[case] input: &str) {
    let result: Result<Account, _> = input.parse();
    assert!(result.is_err(), "{result:?}");
}

#[rstest]
#[case("2014-05-01 open Assets:Cash", "Assets:Cash")]
#[case("2014-05-01 open Liabilities:A", "Liabilities:A")]
#[case("2014-05-01 open Equity:A", "Equity:A")]
#[case("2014-05-01 open Income:A", "Income:A")]
#[case("2014-05-01 open Expenses:A", "Expenses:A")]
#[case("2014-05-01 open Assets:Cash2", "Assets:Cash2")]
#[case("2014-05-01 open Assets:2Cash", "Assets:2Cash")]
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
fn should_parse_open_account_currency(#[case] input: &str, #[case] expected_currencies: &[&str]) {
    let DirectiveContent::Open(open) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    let expected: HashSet<&str> = expected_currencies.iter().copied().collect();
    let actual: HashSet<&str> = open
        .currencies
        .iter()
        .map(beancount_parser::Currency::as_str)
        .collect();
    assert_eq!(actual, expected);
}

#[rstest]
#[case("2014-05-01 open Assets:Checking", None)]
#[case("2014-05-01 open Assets:Checking \"STRICT\"", Some("STRICT"))]
#[case("2014-05-01 open Assets:Checking CHF \"STRICT\"", Some("STRICT"))]
#[case("2014-05-01 open Assets:Checking \t \"STRICT\"", Some("STRICT"))]
#[case(
    "2014-05-01 open Assets:Checking \t \"named \\\"hello\\\"\"",
    Some("named \"hello\"")
)]
fn should_parse_open_account_booking_method(#[case] input: &str, #[case] expected: Option<&str>) {
    let DirectiveContent::Open(open) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(
        open.booking_method
            .as_ref()
            .map(std::convert::AsRef::as_ref),
        expected
    );
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
    let beancount = parse::<f64>(r#"option "He\"llo" "world\"!\"""#).unwrap();
    assert_eq!(beancount.option("He\"llo"), Some("world\"!\""));
}

#[rstest]
fn should_parse_multiple_options_with_same_key() {
    let beancount = parse::<f64>(
        r#"
option "operating_currency" "CHF"
option "operating_currency" "PLN"
"#,
    )
    .unwrap();
    let options: Vec<(&str, &str)> = beancount
        .options
        .iter()
        .map(|opt| (&opt.name[..], &opt.value[..]))
        .collect();
    assert_eq!(
        &options,
        &[("operating_currency", "CHF"), ("operating_currency", "PLN")]
    );
}

#[rstest]
fn should_parse_option_with_comment() {
    let beancount = parse::<f64>(r#"option "Hello" "world!" ; This is great"#).unwrap();
    assert_eq!(beancount.option("Hello"), Some("world!"));
}

#[rstest]
fn should_parse_include() {
    let includes = parse::<f64>(r#"include "./a/path/to/\"file\".beancount""#)
        .unwrap()
        .includes;
    let expected = [Path::new("./a/path/to/\"file\".beancount")];
    assert_eq!(includes, expected);
}

#[rstest]
fn should_parse_include_with_comment() {
    let includes =
        parse::<f64>(r#"include "./a/path/to/file.beancount" ; Everything is in the other file"#)
            .unwrap()
            .includes;
    let expected = &[Path::new("./a/path/to/file.beancount")];
    assert_eq!(&includes, expected);
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
fn should_parse_commodity_that_ends_with_number() {
    let input = "1792-01-01 commodity A1";
    let DirectiveContent::Commodity(commodity) = parse_single_directive(input).content else {
        panic!("was not an commodity directive");
    };
    assert_eq!(commodity.as_str(), "A1");
}

#[rstest]
fn should_parse_event() {
    let input = "2020-12-09 event \"location\" \"New \\\"Metropolis\\\"\"";
    let DirectiveContent::Event(event) = parse_single_directive(input).content else {
        panic!("was not an commodity directive");
    };
    assert_eq!(event.name, "location");
    assert_eq!(event.value, "New \"Metropolis\"");
}

#[rstest]
fn should_parse_price_commodity() {
    let input = "2022-08-26 price VHT          121.03 USD";
    let DirectiveContent::Price(price) = parse_single_directive(input).content else {
        panic!("was not an price directive");
    };
    assert_eq!(price.currency.as_str(), "VHT");
}

#[rstest]
fn should_parse_price_amount() {
    let input = "2022-08-26 price VHT          121.03 USD";
    let DirectiveContent::Price(price) = parse_single_directive(input).content else {
        panic!("was not an price directive");
    };
    assert_eq!(price.amount.value, 121.03);
    assert_eq!(price.amount.currency.as_str(), "USD");
}

#[rstest]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"",
    "title",
    metadata::Value::String("hello".into())
)]
#[case(
    "2022-05-18 commodity CHF\n  value: 1.2",
    "value",
    metadata::Value::Number(1.2)
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n  name: \"world\"",
    "title",
    metadata::Value::String("hello".into())
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n  name: \"world\"",
    "name",
    metadata::Value::String("world".into())
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n  ; Comment\n  name: \"world\"",
    "name",
    metadata::Value::String("world".into())
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n\n  name: \"world\"",
    "name",
    metadata::Value::String("world".into())
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"",
    "title",
    metadata::Value::String("hello".into())
)]
#[case(
    "2022-05-18 *\n  goodTitle: \"Hello world!\"",
    "goodTitle",
    metadata::Value::String("Hello world!".into())
)]
#[case(
    "2022-05-18 *\n  goodTitle: \"Hello \\\"world\\\"!\"",
    "goodTitle",
    metadata::Value::String("Hello \"world\"!".into())
)]
#[case(
    "2022-05-18 *\n  good-title: \"Hello world!\"",
    "good-title",
    metadata::Value::String("Hello world!".into())
)]
#[case(
    "2022-05-18 *\n  good_title: \"Hello world!\"",
    "good_title",
    metadata::Value::String("Hello world!".into())
)]
#[case(
    "2022-05-18 *\n  good_title2: \"Hello world!\"",
    "good_title2",
    metadata::Value::String("Hello world!".into())
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"\n  Assets:Cash 10 CHF",
    "title",
    metadata::Value::String("hello".into())
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"\n  Assets:Cash 10 CHF ; With comment",
    "title",
    metadata::Value::String("hello".into())
)]
fn should_parse_metadata_entry(
    #[case] input: &str,
    #[case] key: &str,
    #[case] expected_value: metadata::Value<f64>,
) {
    let metadata = parse_single_directive(input).metadata;
    assert_eq!(metadata.get(key), Some(&expected_value));
}

#[rstest]
fn should_parse_metadata_currency() {
    let metadata = parse_single_directive("2023-05-27 *\n foo: CHF").metadata;
    let Some(metadata::Value::Currency(currency)) = metadata.get("foo") else {
        panic!("was not a currency: {metadata:?}");
    };
    assert_eq!(currency.as_str(), "CHF");
}

#[rstest]
fn should_reject_invalid_input(
    #[values(
        "2023-06-18 \"Hello\"",
        "14-05-01 open Assets:Cash",
        "2014-05-05 open Assets",
        "2014-05-05 open Assets:hello",
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
        "2014-05-01 open Assets:Checking US-",
        "2014-05-01 open Assets:Checking -US",
        "2014-05-01close Assets:Cash",
        "2014-05-01 closeAssets:Cash",
        "2014-05-01 closeAssets:Cash",
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
        "2020-12-09 event",
        "2022-08-26price VHT  121.03 USD",
        "2022-08-26 priceVHT  121.03 USD",
        "2022-08-26 price VHT121.03 USD",
        "2022-08-26 price VHT 121.03USD",
        "2022-08-26 price VHT",
        "2022-08-26 price 121.03 USD",
        "2014-06-01 pad Assets:BofA:CheckingEquity:Opening-Balances",
        "2014-06-01 padAssets:BofA:Checking Equity:Opening-Balances",
        r#"include"./a/path/to/file.beancount""#
    )]
    input: &str,
) {
    println!("{input}");
    let result = parse::<f64>(input);
    assert!(result.is_err(), "{result:#?}");
}

#[rstest]
fn error_should_contain_relevant_line_number() {
    let input = r#"
2000-01-01 open Assets:AccountsReceivable:Michael  USD
2000-01-01 open Liabilities:CreditCard:CapitalOne

2014-10-05 * "Costco" "Shopping for birthday"
  Liabilities:CreditCard:CapitalOne         -45.00    USD
  Assets:AccountsReceivable:Michael

2014-10-05 * oops

2000-11-01 close Liabilities:CreditCard:CapitalOne"#
        .trim();

    let error_line = parse::<f64>(input).unwrap_err().line_number();
    assert_eq!(error_line, 8);
}

#[rstest]
fn directive_should_contain_relevant_line_number() {
    let input = r#"
2000-01-01 open Assets:AccountsReceivable:Michael  USD
2000-01-01 open Liabilities:CreditCard:CapitalOne

2014-10-05 * "Costco" "Shopping for birthday"
  Liabilities:CreditCard:CapitalOne         -45.00    USD
  Assets:AccountsReceivable:Michael

2000-11-01 close Liabilities:CreditCard:CapitalOne"#
        .trim();

    let line_numbers: Vec<_> = parse::<f64>(input)
        .unwrap()
        .directives
        .into_iter()
        .map(|d| d.line_number)
        .collect();
    assert_eq!(line_numbers, vec![1, 2, 4, 8]);
}

fn parse_single_directive(input: &str) -> Directive<f64> {
    let directives = input
        .parse::<BeancountFile<f64>>()
        .expect("parsing should succeed")
        .directives;
    assert_eq!(
        directives.len(),
        1,
        "unexpected number of directives: {directives:?}"
    );
    directives.into_iter().next().unwrap()
}
