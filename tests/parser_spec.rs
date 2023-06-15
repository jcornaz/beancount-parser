use std::{collections::HashSet, path::Path};

use beancount_parser_2::{parse, Directive, DirectiveContent, MetadataValue};
use rstest::rstest;

const COMMENTS: &str = include_str!("samples/comments.beancount");
const SIMPLE: &str = include_str!("samples/simple.beancount");
const OFFICIAL: &str = include_str!("samples/official.beancount");

#[rstest]
fn should_succeed_for_valid_input(#[values("", "\n", COMMENTS, SIMPLE, OFFICIAL)] input: &str) {
    parse::<f64>(input).expect("parsing should succeed");
}

#[rstest]
#[case("", 0)]
#[case(SIMPLE, 10)]
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
fn should_parse_balance_assertion_account(#[case] input: &str, #[case] exepected: &str) {
    let DirectiveContent::Balance(assertion) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    assert_eq!(assertion.account.as_str(), exepected);
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
    "2020-04-10 balance Assets:US:BofA:Checking        2473 USD",
    2473,
    "USD"
)]
fn should_parse_balance_assertion_amount(
    #[case] input: &str,
    #[case] expected_value: impl Into<f64>,
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
fn should_parse_open_account_currency(#[case] input: &str, #[case] exepcted_currencies: &[&str]) {
    let DirectiveContent::Open(open) = parse_single_directive(input).content else {
        panic!("was not an open directive");
    };
    let expected: HashSet<&str> = exepcted_currencies.iter().copied().collect();
    let actual: HashSet<&str> = open.currencies.iter().map(|c| c.as_str()).collect();
    assert_eq!(actual, expected);
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
    let beancount = parse::<f64>(r#"option "Hello" "world!""#).unwrap();
    assert_eq!(beancount.option("Hello"), Some("world!"));
}

#[rstest]
fn should_parse_option_with_comment() {
    let beancount = parse::<f64>(r#"option "Hello" "world!" ; This is great"#).unwrap();
    assert_eq!(beancount.option("Hello"), Some("world!"));
}

#[rstest]
fn should_parse_include() {
    let includes = parse::<f64>(r#"include "./a/path/to/file.beancount""#)
        .unwrap()
        .includes()
        .collect::<HashSet<_>>();
    let expected: HashSet<_> = [Path::new("./a/path/to/file.beancount")].into();
    assert_eq!(includes, expected);
}

#[rstest]
fn should_parse_include_with_comment() {
    let includes =
        parse::<f64>(r#"include "./a/path/to/file.beancount" ; Everything is in the other file"#)
            .unwrap()
            .includes()
            .collect::<HashSet<_>>();
    let expected: HashSet<_> = [Path::new("./a/path/to/file.beancount")].into();
    assert_eq!(includes, expected);
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
    MetadataValue::String("hello")
)]
#[case(
    "2022-05-18 commodity CHF\n  value: 1.2",
    "value",
    MetadataValue::Number(1.2)
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n  name: \"world\"",
    "title",
    MetadataValue::String("hello")
)]
#[case(
    "2022-05-18 open Assets:Cash\n  title: \"hello\"\n  name: \"world\"",
    "name",
    MetadataValue::String("world")
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"",
    "title",
    MetadataValue::String("hello")
)]
#[case(
    "2022-05-18 *\n  goodTitle: \"Hello world!\"",
    "goodTitle",
    MetadataValue::String("Hello world!")
)]
#[case(
    "2022-05-18 *\n  good-title: \"Hello world!\"",
    "good-title",
    MetadataValue::String("Hello world!")
)]
#[case(
    "2022-05-18 *\n  good_title: \"Hello world!\"",
    "good_title",
    MetadataValue::String("Hello world!")
)]
#[case(
    "2022-05-18 *\n  good_title2: \"Hello world!\"",
    "good_title2",
    MetadataValue::String("Hello world!")
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"\n  Assets:Cash 10 CHF",
    "title",
    MetadataValue::String("hello")
)]
#[case(
    "2022-05-18 * \"a transaction\"\n  title: \"hello\"\n  Assets:Cash 10 CHF ; With comment",
    "title",
    MetadataValue::String("hello")
)]
fn should_parse_metadata_entry(
    #[case] input: &str,
    #[case] key: &str,
    #[case] expected_value: MetadataValue<'static, f64>,
) {
    let metdata = parse_single_directive(input).metadata;
    assert_eq!(metdata.get(key), Some(&expected_value));
}

#[rstest]
fn should_parse_metadata_currency() {
    let metadata = parse_single_directive("2023-05-27 *\n foo: CHF").metadata;
    let Some(MetadataValue::Currency(currency)) = metadata.get("foo") else {
        panic!("was not a currency: {metadata:?}");
    };
    assert_eq!(currency.as_str(), "CHF");
}

#[rstest]
fn should_reject_invalid_input(
    #[values(
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
        "2014-05-01 open Assets:Checking US2",
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

fn parse_single_directive(input: &str) -> Directive<'_, f64> {
    let directives = parse(input).expect("parsing should succeed").directives;
    assert_eq!(
        directives.len(),
        1,
        "unexepcted number of directives: {:?}",
        directives
    );
    directives.into_iter().next().unwrap()
}
