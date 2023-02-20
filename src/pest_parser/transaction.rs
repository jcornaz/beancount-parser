use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::pest_parser::{build_account, build_date, Pair, Rule};
use crate::transaction::{Flag, Posting};
use crate::{Amount, Transaction};

pub(super) fn build(pair: Pair<'_>) -> Transaction<'_> {
    let mut inner = pair.into_inner();
    let date = build_date(inner.next().expect("no date in transaction"));
    let mut flag = None;
    let mut payee = None;
    let mut narration = None;
    let mut postings = Vec::new();
    for pair in inner {
        match pair.as_rule() {
            Rule::transaction_flag => flag = Some(build_flag(pair)),
            Rule::payee => payee = pair.into_inner().next().map(|p| p.as_str().into()),
            Rule::narration => narration = pair.into_inner().next().map(|p| p.as_str().into()),
            Rule::postings => postings = pair.into_inner().map(build_posting).collect(),
            _ => (),
        }
    }
    Transaction {
        date,
        flag,
        payee,
        narration,
        tags: vec![],
        comment: None,
        metadata: HashMap::default(),
        postings,
    }
}

fn build_flag(pair: Pair<'_>) -> Flag {
    match pair.as_str() {
        "*" => Flag::Cleared,
        "!" => Flag::Pending,
        _ => unreachable!("Invalid transaction flag"),
    }
}

fn build_posting(pair: Pair<'_>) -> Posting<'_> {
    let mut inner = pair.into_inner();
    let account = build_account(inner.next().expect("no account in posting"));
    let amount = inner.next().map(build_amount);
    Posting {
        flag: None,
        account,
        price: None,
        lot_attributes: None,
        comment: None,
        amount,
    }
}

fn build_amount(pair: Pair<'_>) -> Amount<'_> {
    let mut inner = pair.into_inner();
    let value: Decimal = inner
        .next()
        .expect("no value in amount")
        .as_str()
        .parse()
        .expect("invalid value");
    let currency = inner.next().expect("no currency in amount").as_str();
    Amount::new(value, currency)
}
