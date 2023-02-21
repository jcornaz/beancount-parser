#![cfg(all(test, feature = "unstable"))]

use pest::Parser as Parse;
use pest_derive::Parser;

use crate::{account, Close, Date, Directive, Open, Transaction};

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "beancount.pest"]
struct Parser;

pub(crate) type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn parse(input: &str) -> Result<impl Iterator<Item = Directive<'_>>, Box<dyn std::error::Error>> {
    Ok(Parser::parse(Rule::beancount_file, input)?
        .next()
        .expect("no root rule")
        .into_inner()
        .filter_map(|p| match p.as_rule() {
            Rule::transaction => Some(Directive::Transaction(Transaction::from_pair(p))),
            Rule::open => Some(Directive::Open(Open::from_pair(p))),
            Rule::close => Some(Directive::Close(Close::from_pair(p))),
            _ => None,
        }))
}
