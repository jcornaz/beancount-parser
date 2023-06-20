//! This example is a cli tool that take some files as argument, parse them and spit out the balance of each accounts
//!
//! To try it out run `cargo run --example balance -- tests/samples/official.beancount`
//! Or, use your own ledger: `cargo run --example balance -- $LEDGER_PATH`
//!
//! This example should play well with grep: `cargo run --example balance -- $LEDGER_PATH | grep Assets`

use std::{
    cmp::Ordering, collections::HashMap, env::args, error::Error, fs::File, io::Read, path::PathBuf,
};

use rust_decimal::Decimal;

use beancount_parser_2::{Account, Amount, Currency, Directive, DirectiveContent, Transaction};

type Report = HashMap<Account, HashMap<Currency, Decimal>>;

fn main() -> Result<(), Box<dyn Error>> {
    let files: Vec<PathBuf> = args().skip(1).map(Into::into).collect();
    let directives = load_from_files(files)?;
    let report = build_report(directives);
    print(&report);
    Ok(())
}

fn load_from_files(mut files: Vec<PathBuf>) -> Result<Vec<Directive<Decimal>>, Box<dyn Error>> {
    let mut directives = Vec::<Directive<Decimal>>::new();
    let mut input = String::new();
    while let Some(path) = files.pop() {
        println!("read: {path:?}");
        input.clear();
        File::open(&path)?.read_to_string(&mut input)?;
        let file = beancount_parser_2::parse::<Decimal>(&input)?;
        files.extend(file.includes.into_iter().map(|include| {
            if include.is_relative() {
                path.parent().unwrap().join(include)
            } else {
                include
            }
        }));
        directives.extend(file.directives);
    }
    Ok(directives)
}

fn compare_directives<D>(a: &Directive<D>, b: &Directive<D>) -> Ordering {
    a.date
        .cmp(&b.date)
        .then_with(|| match (&a.content, &b.content) {
            (DirectiveContent::Balance(_), DirectiveContent::Transaction(_)) => Ordering::Less,
            (DirectiveContent::Transaction(_), DirectiveContent::Balance(_)) => Ordering::Greater,
            _ => Ordering::Equal,
        })
}

fn build_report(mut directives: Vec<Directive<Decimal>>) -> Report {
    directives.sort_by(compare_directives);
    let mut report = Report::new();
    for directive in directives {
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
