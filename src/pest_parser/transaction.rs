use crate::pest_parser::{build_account, build_date, Pair, Rule};
use crate::transaction::{Flag, Posting};
use crate::Transaction;
use std::collections::HashMap;

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
    Posting {
        flag: None,
        account: build_account(pair.into_inner().next().expect("No account")),
        price: None,
        lot_attributes: None,
        comment: None,
        amount: None,
    }
}
