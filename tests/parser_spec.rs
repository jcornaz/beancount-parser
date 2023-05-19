use beancount_parser_2::{metadata, parse, Decimal, Directive, DirectiveContent};
use rstest::rstest;

const COMMENTS: &str = include_str!("samples/comments.beancount");
const SIMPLE: &str = include_str!("samples/simple.beancount");
// TODO const OFFICIAL: &str = include_str!("samples/official.beancount");

#[rstest]
fn should_succeed_for_valid_input(#[values("", "\n", COMMENTS, SIMPLE)] input: &str) {
    parse(input).expect("parsing should succeed");
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
    assert_eq!(price.amount.value, Decimal::new(12103, 2));
    assert_eq!(price.amount.currency.as_str(), "USD");
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
        "2022-08-26 price 121.03 USD"
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
