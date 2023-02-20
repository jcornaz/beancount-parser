#![cfg(all(test, feature = "unstable"))]

use pest::Parser as Parse;
use pest_derive::Parser;

use crate::{account, Account, Close, Date, Directive, Open};

mod transaction;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "beancount.pest"]
struct Parser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn parse(input: &str) -> Result<impl Iterator<Item = Directive<'_>>, Box<dyn std::error::Error>> {
    Ok(Parser::parse(Rule::beancount_file, input)?
        .next()
        .expect("no root rule")
        .into_inner()
        .filter_map(|p| match p.as_rule() {
            Rule::transaction => Some(Directive::Transaction(transaction::build(p))),
            Rule::open => Some(Directive::Open(build_open_directive(p))),
            Rule::close => Some(Directive::Close(build_close_directive(p))),
            _ => None,
        }))
}

fn build_open_directive(pair: Pair<'_>) -> Open<'_> {
    let mut inner = pair.into_inner();
    let date = build_date(inner.next().expect("no date in open directive"));
    let account = build_account(inner.next().expect("no account in open directive"));
    let currencies = inner.map(|c| c.as_str()).collect();
    Open {
        date,
        account,
        currencies,
    }
}

fn build_close_directive(pair: Pair<'_>) -> Close<'_> {
    let mut inner = pair.into_inner();
    let date = build_date(inner.next().expect("no date in close directive"));
    let account = build_account(inner.next().expect("no account in close directive"));
    Close { date, account }
}

fn build_date(pair: Pair<'_>) -> Date {
    let mut inner = pair.into_inner();
    let year = inner
        .next()
        .and_then(|y| y.as_str().parse().ok())
        .expect("invalid year");
    let month = inner
        .next()
        .and_then(|m| m.as_str().parse().ok())
        .expect("invalid month");
    let day = inner
        .next()
        .and_then(|d| d.as_str().parse().ok())
        .expect("invalid day");
    Date::new(year, month, day)
}

fn build_account(pair: Pair<'_>) -> Account<'_> {
    let mut inner = pair.into_inner();
    let type_ = match inner.next().expect("no account type in account").as_str() {
        "Assets" => account::Type::Assets,
        "Liabilities" => account::Type::Liabilities,
        "Expenses" => account::Type::Expenses,
        "Income" => account::Type::Income,
        "Equity" => account::Type::Equity,
        _ => unreachable!("invalid account type"),
    };
    let components = inner.map(|c| c.as_str()).collect();
    Account { type_, components }
}
