use beancount_parser::transaction::Posting;

use crate::assert_single_transaction;

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

fn make_transaction_from_posting(posting_input: &str) -> String {
    format!("2022-03-03 txn \"\"\n  {posting_input}")
}

fn assert_posting(input: &str) -> Posting<'_> {
    let transaction = assert_single_transaction(input);
    let postings = transaction.postings();
    assert_eq!(postings.len(), 1);
    postings[0].clone()
}
