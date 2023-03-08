use rstest::rstest;

use beancount_parser::transaction::{Flag, Posting, PriceType};
use beancount_parser::{Amount, Date, Parser, Transaction};

use crate::utils::assert_single_directive;

mod utils;

#[rstest]
fn simple_transaction() {
    let input = r#"2022-09-16 * "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.date(), Date::new(2022, 9, 16));
    assert_eq!(transaction.narration(), Some(r#"Hello "world""#));
    assert_eq!(transaction.postings().len(), 2);
    assert_eq!(transaction.flag(), Some(Flag::Cleared));
    assert_eq!(transaction.payee(), None);
    assert_eq!(transaction.comment(), None);
    assert_eq!(transaction.tags().len(), 0);
}

#[test]
fn transaction_without_posting() {
    let input = r#"2022-01-01 * "Hello \"world\"""#;
    let transaction = assert_single_transaction(input);
    assert!(transaction.postings().is_empty());
}

#[test]
fn transaction_without_description() {
    let input = r#"2022-01-01 *
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
    let transaction = assert_single_transaction(input);
    assert!(transaction.narration().is_none());
}

#[test]
#[cfg(feature = "unstable")]
fn should_parse_metadata() {
    use beancount_parser::metadata;
    let input = r#"2022-01-01 *
            abc: Assets:Unknown
            def: 3 USD
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
    let transaction = assert_single_transaction(input);
    let Some(metadata::Value::Account(account)) = transaction.metadata().get(&String::from("abc")) else { panic!("unexpected metadata") };
    assert_eq!(&account.to_string(), "Assets:Unknown");
}

#[test]
fn should_succeed_with_metadata() {
    let input = r#"2022-01-01 *
            abc: Assets:Unknown
            def: 3 USD
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.postings().len(), 2, "{transaction:?}");
}

#[test]
fn transaction_with_payee() {
    let input = r#"2022-01-01 * "me" "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.payee(), Some("me"));
}

#[test]
fn transaction_with_exclamation_mark() {
    let input = r#"2022-01-01 ! "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.flag(), Some(Flag::Pending));
}

#[test]
fn transaction_without_flag() {
    let input = r#"2022-01-01 txn "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
    let transaction = assert_single_transaction(input);
    assert!(transaction.flag().is_none());
}

#[test]
fn transaction_with_one_tag() {
    let input = r#"2022-01-01 txn "Hello \"world\"" #hello-world"#;
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.tags(), ["hello-world"]);
}

#[test]
fn transaction_with_multiple_tags() {
    let input = r#"2022-01-01 txn "Hello \"world\"" #that #is #cool"#;
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.tags(), ["that", "is", "cool"]);
}

#[test]
fn transaction_with_comment() {
    let input = r#"2022-01-01 txn "Hello \"world\"" ; And a comment!"#;
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.comment(), Some("And a comment!"));
}

#[test]
fn pushtags_adds_tag_to_next_transaction() {
    let input = "pushtag #hello\n2022-10-20 txn";
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.tags(), &["hello"]);
}

#[test]
fn multiple_pushtags_add_tags_to_next_transaction() {
    let input = "pushtag #hello\npushtag #world\n2022-10-20 txn";
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.tags(), &["hello", "world"]);
}

#[test]
fn poptag_removes_tag_from_stack() {
    let input = "pushtag #hello\npoptag #hello\n2022-10-20 txn";
    let transaction = assert_single_transaction(input);
    assert!(transaction.tags().is_empty());
}

#[test]
fn poptag_removes_only_concerned_tag_from_stack() {
    let input = "pushtag #hello\npushtag #world\npoptag #hello\n2022-10-20 txn";
    let transaction = assert_single_transaction(input);
    assert_eq!(transaction.tags(), &["world"]);
}

#[test]
fn simple_posting() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF");
    let posting = assert_posting(&input);
    assert_eq!(&posting.account().to_string(), "Assets:A:B",);
    assert_eq!(
        posting.amount().unwrap().value().try_into_f64().unwrap(),
        10.0
    );
    assert_eq!(posting.amount().unwrap().currency(), "CHF");
    assert!(posting.price().is_none());
    assert!(posting.cost().is_none());
    assert!(posting.comment().is_none());
}

#[test]
fn without_amount() {
    let input = make_transaction_from_posting("Assets:A:B");
    let posting = assert_posting(&input);
    assert!(posting.amount().is_none());
}

#[test]
fn with_price() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF @ 1 EUR");
    let posting = assert_posting(&input);
    let (price_type, amount) = posting.price().unwrap();
    assert_eq!(price_type, PriceType::Unit);
    assert_eq!(amount.value().try_into_f64().unwrap(), 1.0);
    assert_eq!(amount.currency(), "EUR");
}

#[test]
fn with_total_price() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF @@ 9 EUR");
    let posting = assert_posting(&input);
    let (price_type, amount) = posting.price().unwrap();
    assert_eq!(price_type, PriceType::Total);
    assert_eq!(amount.value().try_into_f64().unwrap(), 9.0);
    assert_eq!(amount.currency(), "EUR");
}

#[rstest]
fn with_cost(#[values("Assets:A:B 10 CHF {1 EUR}", "Assets:A:B 10 CHF { 1 EUR }")] input: &str) {
    let input = make_transaction_from_posting(input);
    let posting = assert_posting(&input);
    let cost = posting.cost().unwrap();
    assert_eq!(cost.value().try_into_f64().unwrap(), 1.0);
    assert_eq!(cost.currency(), "EUR");
}

#[rstest]
fn with_empty_cost_and_nonempty_price(
    #[values("Assets:A:B -10 CHF {} @ 1 EUR", "Assets:A:B -10 CHF { } @ 1 EUR")] input: &str,
) {
    let input = make_transaction_from_posting(input);
    let posting = assert_posting(&input);
    assert!(posting.cost().is_none());
    let (price, amount) = posting.price().unwrap();
    assert_eq!(price, PriceType::Unit);
    assert_eq!(amount.value().try_into_f64().unwrap(), 1.0);
    assert_eq!(amount.currency(), "EUR");
}

#[test]
fn with_cost_and_date() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF {1 EUR , 2022-10-14}");
    let posting = assert_posting(&input);
    assert_eq!(
        posting.cost().and_then(|a| a.value().try_into_f64().ok()),
        Some(1.0)
    );
    assert_eq!(posting.cost().map(Amount::currency), Some("EUR"));
}

#[test]
fn with_cost_and_date_and_label() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF {1 EUR, 2022-10-14, \"label\"}");
    let posting = assert_posting(&input);
    assert_eq!(
        posting.cost().and_then(|a| a.value().try_into_f64().ok()),
        Some(1.0)
    );
    assert_eq!(posting.cost().map(Amount::currency), Some("EUR"));
}

#[test]
fn with_cost_and_no_date_and_label() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF {1 EUR, \"label\"}");
    let posting = assert_posting(&input);
    assert_eq!(
        posting.cost().and_then(|a| a.value().try_into_f64().ok()),
        Some(1.0)
    );
    assert_eq!(posting.cost().map(Amount::currency), Some("EUR"));
}

#[test]
fn with_cost_and_price() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF {2 USD} @ 1 EUR");
    let posting = assert_posting(&input);
    assert_eq!(
        posting.cost().and_then(|a| a.value().try_into_f64().ok()),
        Some(2.0)
    );
    assert_eq!(posting.cost().map(Amount::currency), Some("USD"));
    let Some((PriceType::Unit, price)) = posting.price() else { panic!("unexpected price in {posting:?}") };
    assert_eq!(price.value().try_into_f64(), Ok(1.0));
    assert_eq!(price.currency(), "EUR");
}

#[test]
fn with_flag() {
    let input = make_transaction_from_posting("! Assets:A 1 EUR");
    let posting = assert_posting(&input);
    assert_eq!(posting.flag(), Some(Flag::Pending));
}

#[test]
fn with_comment() {
    let input = make_transaction_from_posting("Assets:A:B 10 CHF ; Cool!");
    let posting = assert_posting(&input);
    assert_eq!(posting.comment(), Some("Cool!"));
}

#[rstest]
fn failures(
    #[values(
        r#"2022-01-01 *"hello""#,
        r#"2022-01-01 * "hello" Assets:A 10 CHF"#,
        "2022-01-01 ! test"
    )]
    input: &str,
) {
    let item = Parser::new(input).next().expect("nothing found");
    assert!(item.is_err(), "{item:?}");
}

fn assert_single_transaction(input: &str) -> Transaction<'_> {
    assert_single_directive(input)
        .into_transaction()
        .expect("was not a transaction")
}

fn make_transaction_from_posting(posting_input: &str) -> String {
    format!("2022-03-03 txn \"\"\n  {posting_input}")
}

fn assert_posting(input: &str) -> Posting<'_> {
    let transaction = assert_single_transaction(input);
    let postings = transaction.postings();
    assert_eq!(postings.len(), 1);
    postings[0].clone()
}
