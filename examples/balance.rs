//! This example is a cli tool that take a beancount file from stdin and spit out the balance of each accounts
//!
//! To try it out run `bean-example | cargo run --example balance`
//! Or, use your own ledger: `cat $LEDGER_PATH | cargo run --example balance`
//!
//! This example should play well with grep: `cat $LEDGER_PATH | cargo run --example balance | grep Assets`
//!
//! Important note: The current implementation doesn't handle all edge-cases (yet).
//! In particular it:
//! * Ignores postings without amount
//! * Ignores includes, pad and balance directives

use std::collections::HashMap;
use std::io::{stdin, Read};
use std::process::exit;

use beancount_parser_2::{Account, BeancountFile, Currency, DirectiveContent};

type Report<'a> = HashMap<Account<'a>, HashMap<Currency<'a>, rust_decimal::Decimal>>;

fn main() {
    let mut input = String::new();
    stdin()
        .read_to_string(&mut input)
        .expect("cannot read from stdin");
    let beancount = match beancount_parser_2::parse::<rust_decimal::Decimal>(&input) {
        Ok(beancount) => beancount,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };
    let report = build_report(beancount);
    print(&report);
}

fn build_report(beancount: BeancountFile<rust_decimal::Decimal>) -> Report {
    beancount
        .directives
        .into_iter()
        .filter_map(|d| match d.content {
            DirectiveContent::Transaction(trx) => Some(trx),
            _ => None,
        })
        .flat_map(|trx| trx.postings.into_iter())
        .filter_map(|posting| Some((posting.account, posting.amount?)))
        .fold(Report::new(), |mut report, (account, amount)| {
            let value = report
                .entry(account)
                .or_default()
                .entry(amount.currency)
                .or_default();
            *value += amount.value;
            report
        })
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
                println!(
                    "{account:50} {value:>15.2} {currency}",
                    currency = currency.as_str()
                );
            });
    }
}
