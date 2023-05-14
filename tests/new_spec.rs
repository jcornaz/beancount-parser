use beancount_parser::{
    account,
    transaction::{Flag, Posting},
    Date, Directive, Parser,
};

use rstest::rstest;

const COMMENTS: &str = include_str!("../tests/samples/comments.beancount");
const SIMPLE: &str = include_str!("../tests/samples/simple.beancount");
const OFFICIAL: &str = include_str!("../tests/samples/official.beancount");

#[rstest]
fn successful_parse(#[values("", " ", " \n ", " \t ", COMMENTS, SIMPLE, OFFICIAL)] input: &str) {
    if let Err(err) = Parser::new(input).collect::<Result<Vec<_>, _>>() {
        panic!("{err:?}");
    }
}

#[rstest]
fn comments(
    #[values(
        "",
        "\n",
        "\n\n\r\n",
        "2016 - 11 - 28 close Liabilities:CreditCard:CapitalOne",
        "Hello world",
        "* Banking",
        "** Bank of America",
        ";; Transactions follow …",
        "; foo bar",
        ";"
    )]
    input: &str,
) {
    let len = Parser::new(input)
        .collect::<Result<Vec<_>, _>>()
        .expect("should successfully parse input")
        .len();
    assert_eq!(len, 0);
}

#[rstest]
#[case(
    "2016-11-28 close Liabilities:CreditCard:CapitalOne",
    Date::new(2016, 11, 28)
)]
#[case("2022-12-31 open Assets:A", Date::new(2022, 12, 31))]
#[case("2000-01-01 txn", Date::new(2000, 1, 1))]
#[case("2000-01-02 * \"Groceries\"", Date::new(2000, 1, 2))]
#[case("2000-01-03 * \"Store\" \"Groceries\"", Date::new(2000, 1, 3))]
#[case("2000-01-04 *", Date::new(2000, 1, 4))]
#[case("2000-01-05 !", Date::new(2000, 1, 5))]
#[case(
    "2020-01-02 balance Assets:US:BofA:Checking        3467.65 USD",
    Date::new(2020, 1, 2)
)]
fn parse_date(#[case] input: &str, #[case] expected: Date) {
    let directive = parse_single_directive(input);
    assert_eq!(directive.date(), Some(expected));
}

#[rstest]
#[case("2000-01-01 txn", None)]
#[case("2000-01-01 txn \"Store\"", None)]
#[case("2000-01-01 *", Some(Flag::Cleared))]
#[case("2000-01-01 * \"Store\"", Some(Flag::Cleared))]
#[case("2000-01-01 !", Some(Flag::Pending))]
#[case("2000-01-01 ! \"Store\"", Some(Flag::Pending))]
fn parse_transaction_flag(#[case] input: &str, #[case] expected: Option<Flag>) {
    let transaction = parse_single_directive(input).into_transaction().unwrap();
    assert_eq!(transaction.flag(), expected);
}

#[rstest]
#[case("2022-02-12 txn", None, None)]
#[case("2022-02-12  txn", None, None)]
#[case("2022-02-12 *", None, None)]
#[case("2022-02-12  *  ", None, None)]
#[case("2022-02-12 txn \"Hello\"", None, Some("Hello"))]
#[case("2022-02-12   txn  \"Hello\"", None, Some("Hello"))]
#[case("2022-02-12 * \"Hello\"", None, Some("Hello"))]
#[case("2022-02-12 txn \"Hello\" \"World\"", Some("Hello"), Some("World"))]
#[case("2022-02-12 txn \"Hello\" \t \"World\"", Some("Hello"), Some("World"))]
#[case("2022-02-12 ! \"Hello\" \"World\"", Some("Hello"), Some("World"))]
fn parse_transaction_payee_and_description(
    #[case] input: &str,
    #[case] expected_payee: Option<&str>,
    #[case] expected_narration: Option<&str>,
) {
    let transaction = parse_single_directive(input).into_transaction().unwrap();
    assert_eq!(transaction.payee(), expected_payee);
    assert_eq!(transaction.narration(), expected_narration);
}

#[rstest]
#[case(r#"2022-02-12 txn"#, &[])]
#[case(r#"2022-02-12 txn #hello"#, &["hello"])]
#[case(r#"2022-02-12 txn "Payee" "Narration" #hello"#, &["hello"])]
#[case(r#"2022-02-12 txn "Payee" "Narration" #Hello #world"#, &["Hello", "world"])]
#[case(r#"2020-11-24 * "Legal Seafood" "" #trip-boston-2020"#, &["trip-boston-2020"])]
fn parse_transaction_tags(#[case] input: &str, #[case] expected: &[&str]) {
    let transaction = parse_single_directive(input).into_transaction().unwrap();
    assert_eq!(transaction.tags(), expected);
}

#[rstest]
#[case("2022-02-12 txn", &[])]
#[case("2022-02-12 txn\n  Assets:Hello", &["Assets:Hello"])]
#[case("2022-02-12 txn\nAssets:Hello", &["Assets:Hello"])]
#[case("2022-02-12 txn\n\tAssets:Hello", &["Assets:Hello"])]
#[case("2022-02-12 txn\n  Assets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
#[case("2022-02-12 txn ; Hello\n  Assets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
#[case("2022-02-12 txn; Hello\n  Assets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
#[case("2022-02-12 txn ; Hello\nAssets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
#[case("2022-02-12 txn\nAssets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
#[case("2020-11-24 * \"Legal Seafood\" \"\" #trip-boston-2020\n  Liabilities:US:Chase:Slate  -40.15 USD\n  Expenses:Food:Restaurant  40.15 USD", &["Liabilities:US:Chase:Slate", "Expenses:Food:Restaurant"])]
fn parse_posting_accounts(#[case] input: &str, #[case] expected: &[&str]) {
    let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
    let transaction = parse_single_directive(input).into_transaction().unwrap();
    let actual: Vec<String> = transaction
        .postings()
        .iter()
        .map(Posting::account)
        .map(ToString::to_string)
        .collect();
    assert_eq!(actual, expected);
}

#[rstest]
#[case("Assets:Hello", None)]
#[case("  Assets:Hello", None)]
#[case("* Assets:Hello", Some(Flag::Cleared))]
#[case("  * Assets:Hello", Some(Flag::Cleared))]
#[case("  *  Assets:Hello", Some(Flag::Cleared))]
#[case("! Assets:Hello", Some(Flag::Pending))]
#[case("  ! Assets:Hello", Some(Flag::Pending))]
#[case("  !  Assets:Hello", Some(Flag::Pending))]
fn parse_posting_flag(#[case] input: &str, #[case] expected: Option<Flag>) {
    let input = format!("2022-02-23 txn\n{input}");
    let transaction = parse_single_directive(&input).into_transaction().unwrap();
    let posting = &transaction.postings()[0];
    assert_eq!(posting.flag(), expected);
}

#[rstest]
#[case("Assets:Hello", None)]
#[case("Assets:Hello ; Hello", Some("Hello"))]
#[case("Assets:Hello 10 CHF ; World", Some("World"))]
#[case("Assets:Hello  10 CHF ; World", Some("World"))]
#[case("Assets:Hello 10 CHF ;;;  World", Some("World"))]
#[case("Assets:Hello 10 CHF; Tadaa", Some("Tadaa"))]
fn parse_posting_comment(#[case] input: &str, #[case] expected: Option<&str>) {
    let input = format!("2022-02-23 txn\n{input}");
    let transaction = parse_single_directive(&input).into_transaction().unwrap();
    let posting = &transaction.postings()[0];
    assert_eq!(posting.comment(), expected);
}

#[rstest]
#[case("2016-11-28 close Assets:Hello", account::Type::Assets)]
#[case("2016-11-28 close Liabilities:Hello", account::Type::Liabilities)]
#[case("2016-11-28 close Expenses:Hello", account::Type::Expenses)]
#[case("2016-11-28 close Income:Hello", account::Type::Income)]
#[case("2016-11-28 close Equity:Hello", account::Type::Equity)]
#[case("2016-11-28 close Equity:Hello ; Foo bar", account::Type::Equity)]
#[case("2016-11-28 close Equity:Hello; Foo bar", account::Type::Equity)]
#[case("2016-11-28 close Equity:Hello;Foo bar", account::Type::Equity)]
#[case("2016-11-28 close Equity:Hello;", account::Type::Equity)]
#[case("2016-11-28  close  Equity:Hello;", account::Type::Equity)]
#[case("2016-11-28\tclose\tEquity:Hello;", account::Type::Equity)]
fn parse_close_directive_account_type(
    #[case] input: &str,
    #[case] expected_account_type: account::Type,
) {
    let directive = parse_single_directive(input);
    let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
    assert_eq!(close.account().type_(), expected_account_type);
}

#[rstest]
#[case("2016-11-28 open Assets:Hello", account::Type::Assets)]
#[case("2016-11-28  open  Assets:Hello", account::Type::Assets)]
#[case("2016-11-28 open Liabilities:Hello", account::Type::Liabilities)]
fn parse_open_directive_account_type(
    #[case] input: &str,
    #[case] expected_account_type: account::Type,
) {
    let directive = parse_single_directive(input);
    let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
    assert_eq!(open.account().type_(), expected_account_type);
}

#[rstest]
#[case("2016-11-28 close Liabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
#[case("2016-11-28 close Assets:Hello", &["Hello"])]
#[case("2016-11-28 close Assets", &[])]
#[case("2016-11-28 close Assets:Hello-World:123", &["Hello-World", "123"])]
#[case("2016-11-28  close\tLiabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
fn parse_close_directive_account_components(
    #[case] input: &str,
    #[case] expected_account_components: &[&str],
) {
    let directive = parse_single_directive(input);
    let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
    assert_eq!(close.account().components(), expected_account_components);
}

#[rstest]
#[case("2016-11-28 open Liabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
#[case("2016-11-28 open Assets:Hello", &["Hello"])]
#[case("2016-11-28 open Assets", &[])]
#[case("2016-11-28 open Assets:Hello-World:123", &["Hello-World", "123"])]
#[case("2016-11-28  open\t\tLiabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
fn parse_open_directive_account_components(
    #[case] input: &str,
    #[case] expected_account_components: &[&str],
) {
    let directive = parse_single_directive(input);
    let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
    assert_eq!(open.account().components(), expected_account_components);
}

#[rstest]
#[case("2016-11-28 open Assets", &[])]
#[case("2016-11-28 open Assets CHF", &["CHF"])]
#[case("2016-11-28  open  Assets CHF", &["CHF"])]
#[case("2016-11-28\topen\tAssets CHF", &["CHF"])]
#[case("2016-11-28 open Assets CHF,EUR", &["CHF", "EUR"])]
#[case("2016-11-28 open Assets CHF , EUR", &["CHF", "EUR"])]
#[case("2016-11-28 open Assets AB-CD, A_2B, A.B, A'B", &["AB-CD", "A_2B", "A.B", "A'B"])]
fn parse_open_directive_currencies(#[case] input: &str, #[case] expected: &[&str]) {
    let directive = parse_single_directive(input);
    let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
    assert_eq!(open.currencies(), expected);
}

#[rstest]
#[case(r#"option "operating_currency" "USD""#, "operating_currency", "USD")]
#[case(r#"option "operating_currency" "USD""#, "operating_currency", "USD")]
#[case(r#"option"operating_currency" "USD""#, "operating_currency", "USD")]
#[case(r#"option "operating_currency""USD""#, "operating_currency", "USD")]
#[case(r#"option  "operating_currency"  "USD""#, "operating_currency", "USD")]
#[case("option\t\"operating_currency\"\t\"USD\"", "operating_currency", "USD")]
#[case(
    r#"option "Can you say hello?" "Hello world!""#,
    "Can you say hello?",
    "Hello world!"
)]
#[cfg(feature = "unstable")]
fn parse_option(#[case] input: &str, #[case] expected_name: &str, #[case] expected_value: &str) {
    let directive = parse_single_directive(input);
    let Directive::Option(option) = directive else { panic!("expected option but was {directive:?}") };
    assert_eq!(option.name(), expected_name);
    assert_eq!(option.value(), expected_value);
}

#[rstest]
#[cfg(feature = "unstable")]
fn parse_event(
    #[values(
        r#"2020-11-23 event "location" "Boston""#,
        r#"2020-11-23  event  "location"  "Boston""#,
        r#"2020-11-23event "location""Boston""#
    )]
    input: &str,
) {
    let directive = parse_single_directive(input);
    let Directive::Event(event) = directive else { panic!("expected event but was {directive:?}") };
    assert_eq!(event.date(), Date::new(2020, 11, 23));
    assert_eq!(event.name(), "location");
    assert_eq!(event.value(), "Boston");
}

#[rstest]
#[ignore = "not implemented"]
fn error_case(
    #[values(
        "2016-11-28closeLiabilities:CreditCard:CapitalOne",
        "2016-11-28 closeLiabilities:CreditCard:CapitalOne",
        "2016-11-28close Liabilities:CreditCard:CapitalOne",
        "2016-11-28 close Liabilities:CreditCard:CapitalOne Oops",
        "2016-11-28 close Oops",
        "2016-11-28openAssets:A",
        "2016-11-28 openAssets:A",
        "2016-11-28open Assets:A",
        "2016-11-28 open Assets:A oops",
        "2016-11-28 open Assets:A 22",
        "2016-11-28 open Oops",
        "2020-01-03price VBMPX 186 USD",
        "2020-01-03 priceVBMPX 186 USD",
        "2020-01-03 price VBMPX186 USD",
        "2020-01-03 price VBMPX",
        "2020-01-03 price ",
        "2022-02-12 txn\n  Assets:Hello  10CHF",
        "2022-02-12 txn\n  Assets:Hello10 CHF",
        "2022-02-12 txn\n  Assets:Hello 1 +  CHF",
        "2022-02-12 txn\n  Assets:Hello 2 *  CHF",
        "2022-02-12 txn\n  Assets:Hello 1 /  CHF",
        "2022-02-12 txn\n  Assets:Hello 1 CHF {",
        "2022-02-12 txn\n  Assets:Hello 1 CHF }",
        "2022-02-12 txn\n  Assets:Hello 1 CHF {,}",
        "2022-02-12 txn\n  Assets:Hello 1 CHF {1 CHF,}",
        "2022-02-12 txn\n  Assets:Hello 1 CHF {,1 CHF}",
        "2022-02-12 txn\n  Assets:Hello 1 CHF @",
        "2022-02-12 txn\n  Assets:Hello 1 CHF @ ",
        "2022-02-12 txn\n  Assets:Hello 1 CHF @@",
        "2022-02-12 txnAssets:Hello 1 /  CHF",
        "2022-02-12 txn oops",
        r#"2022-02-12 txn "hello""world""#,
        r#"2022-02-12 txn"hello" "world""#,
        r#"2022-02-12 txn#Hello"#,
        r#"2022-02-12 txn #Hello#world"#,
        r#"2022-02-12 txn #"#,
        r#"2022-02-12 txn ##"#,
        r#"2022-02-12 txn # #"#,
        "1792-01-01 commodityUSD",
        "1792-01-01commodity USD",
        "1792-01-01 balance",
        "1792-01-01 balance Test",
        "1792-01-01 balance Assets:A",
        "1792-01-01 balance Assets:A",
        "1792-01-01 balanceAssets:A 1 CHF",
        "1792-01-01balance Assets:A 1 CHF",
        r#"2020-11-23event "location" "Boston""#,
        r#"2020-11-23 event"location" "Boston""#,
        r#"2020-11-23 event "location""Boston""#,
        "option",
        "option a b",
        "option \"a\"",
        "option \"a\" b",
        "option\"a\" \"b\"",
        "option \"a\"\"b\""
    )]
    input: &str,
) {
    let result = Parser::new(input).next().unwrap();
    assert!(result.is_err(), "{result:?}");
}

fn parse_single_directive(input: &str) -> Directive<'_> {
    let mut parser = Parser::new(input);
    let directive = parser.next().expect("no directive found").unwrap();
    assert!(parser.next().is_none(), "more than one directive found");
    directive
}
