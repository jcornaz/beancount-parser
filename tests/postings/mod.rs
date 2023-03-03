use beancount_parser::transaction::{Posting, PriceType};

use crate::assert_single_transaction;
use rstest::rstest;

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

fn make_transaction_from_posting(posting_input: &str) -> String {
    format!("2022-03-03 txn \"\"\n  {posting_input}")
}

fn assert_posting(input: &str) -> Posting<'_> {
    let transaction = assert_single_transaction(input);
    let postings = transaction.postings();
    assert_eq!(postings.len(), 1);
    postings[0].clone()
}
