//! This example is a cli tool that take a beancount file from stdin and spit out the balance of each accounts
//!
//! To try it out run `bean-example | cargo run --example balance`
//! Or, use your own ledger: `cat $LEDGER_PATH | cargo run --example balance`
//!
//! This example should play well with grep: `cat $LEDGER_PATH | cargo run --example balance | grep Assets`
//!
//! Important note: The current implementation doesn't handle all edge-cases (yet).
//! In particular it ignores includes, pad and balance directives

use std::{
    collections::HashMap,
    io::{stdin, Read},
    process::exit,
};

use rust_decimal::Decimal;

use beancount_parser_2::{Account, Amount, BeancountFile, Currency, DirectiveContent, Transaction};

type Report<'a> = HashMap<Account<'a>, HashMap<Currency<'a>, Decimal>>;

fn main() {
    let mut input = String::new();
    stdin()
        .read_to_string(&mut input)
        .expect("cannot read from stdin");
    let beancount = match beancount_parser_2::parse::<Decimal>(&input) {
        Ok(beancount) => beancount,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };
    let report = build_report(beancount);
    print(&report);
}

fn build_report(beancount: BeancountFile<Decimal>) -> Report {
    beancount
        .directives
        .into_iter()
        .filter_map(|d| match d.content {
            DirectiveContent::Transaction(trx) => Some(trx),
            _ => None,
        })
        .fold(Report::new(), |mut report, trx| {
            add_trx(&mut report, trx);
            report
        })
}

/// Add a transaction to the report
fn add_trx<'a>(report: &mut Report<'a>, transaction: Transaction<'a, Decimal>) {
    // If there is a posting without amount, then it should be consider as a source account for balancing the transaction
    let source_account = transaction
        .postings
        .iter()
        .find(|p| p.amount.is_none())
        .map(|p| p.account);
    transaction
        .postings
        .into_iter()
        .filter_map(|p| Some((p.account, p.amount?)))
        .for_each(|(account, mut amount)| {
            add_amount(report, account, amount);
            // If there is a source account, then we balance the transaction by removing the amount from the source account
            if let Some(account) = source_account {
                amount.value = -amount.value;
                add_amount(report, account, amount);
            }
        });
}

fn add_amount<'a>(report: &mut Report<'a>, account: Account<'a>, amount: Amount<'a, Decimal>) {
    let value = report
        .entry(account)
        .or_default()
        .entry(amount.currency)
        .or_default();
    *value += amount.value;
}

fn print(report: &Report) {
    let mut accounts: Vec<_> = report.keys().collect();
    accounts.sort();
    for account in accounts {
        report
            .get(account)
            .iter()
            .flat_map(|a| a.iter())
            .for_each(|(currency, value)| {
                println!("{account:50} {value:>15.2} {currency}");
            });
    }
}
