//! This example is a cli tool that take a beancount file from stdin and spit out the balance of each accounts
//!
//! To try it out run `bean-example | cargo run --example balance`
//! Or, use your own ledger: `cat $LEDGER_PATH | cargo run --example balance`
//!
//! This example should play well with grep: `cat $LEDGER_PATH | cargo run --example balance | grep Assets`
//!
//! Important note: The current implementation ignores the 'include' directives

use std::{
    cmp::Ordering,
    collections::HashMap,
    io::{stdin, Read},
    process::exit,
};

use rust_decimal::Decimal;

use beancount_parser_2::{
    Account, Amount, BeancountFile, Currency, Directive, DirectiveContent, Transaction,
};

type Report<'a> = HashMap<Account, HashMap<Currency, Decimal>>;

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

fn compare_directives<'a, D>(a: &Directive<'a, D>, b: &Directive<'a, D>) -> Ordering {
    a.date
        .cmp(&b.date)
        .then_with(|| match (&a.content, &b.content) {
            (DirectiveContent::Balance(_), DirectiveContent::Transaction(_)) => Ordering::Less,
            (DirectiveContent::Transaction(_), DirectiveContent::Balance(_)) => Ordering::Greater,
            _ => Ordering::Equal,
        })
}

fn build_report(mut beancount: BeancountFile<Decimal>) -> Report {
    beancount.directives.sort_by(compare_directives);
    let mut report = Report::new();
    for directive in beancount.directives {
        match directive.content {
            DirectiveContent::Transaction(trx) => add_trx(&mut report, trx),
            DirectiveContent::Balance(bal) => set_balance(&mut report, bal.account, bal.amount),
            _ => (),
        }
    }
    report
}

/// Add a transaction to the report
fn add_trx(report: &mut Report, transaction: Transaction<Decimal>) {
    // If there is a posting without amount, then it should be consider as a source account for balancing the transaction
    let source_account = transaction
        .postings
        .iter()
        .find(|p| p.amount.is_none())
        .map(|p| p.account.clone());
    transaction
        .postings
        .into_iter()
        .filter_map(|p| Some((p.account, p.amount?)))
        .for_each(|(account, mut amount)| {
            // If there is a source account, then we balance the transaction by removing the amount from the source account
            if let Some(ref source_account) = source_account {
                add_amount(report, account, amount.clone());
                amount.value = -amount.value;
                add_amount(report, source_account.clone(), amount);
            } else {
                add_amount(report, account, amount);
            }
        });
}

fn add_amount(report: &mut Report, account: Account, amount: Amount<Decimal>) {
    let value = report
        .entry(account)
        .or_default()
        .entry(amount.currency)
        .or_default();
    *value += amount.value;
}

fn set_balance(report: &mut Report, account: Account, amount: Amount<Decimal>) {
    let value = report
        .entry(account)
        .or_default()
        .entry(amount.currency)
        .or_default();
    *value = amount.value;
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
