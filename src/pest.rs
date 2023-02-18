#![cfg(test)]

use crate::{account, Account, Close, Date, Directive};
use pest::Parser as Parse;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "beancount.pest"]
struct Parser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn parse(input: &str) -> Result<impl Iterator<Item = Directive<'_>>, Box<dyn std::error::Error>> {
    Ok(Parser::parse(Rule::beancount_file, input)?
        .flatten()
        .filter_map(|p| match p.as_rule() {
            Rule::close_directive => Some(Directive::Close(build_close_directive(p))),
            _ => None,
        }))
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
        .expect("no year in date")
        .as_str()
        .parse()
        .expect("year is not a number");
    let month = inner
        .next()
        .expect("no month in date")
        .as_str()
        .parse()
        .expect("year is not a number");
    let day = inner
        .next()
        .expect("no day in date")
        .as_str()
        .parse()
        .expect("year is not a number");
    Date::new(year, month, day)
}

fn build_account(pair: Pair<'_>) -> Account<'_> {
    let mut inner = pair.into_inner();
    let type_ = match inner.next().expect("no account type in account").as_str() {
        "Liabilities" => account::Type::Liabilities,
        _ => unreachable!("invalid account type"),
    };
    let components = inner.map(|c| c.as_str()).collect();
    Account { type_, components }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    fn successful_parse(
        #[values("", " ", " \n ", " \t ",
            include_str!("../tests/examples/simple.beancount"),
            include_str!("../tests/examples/official.beancount"),
            include_str!("../tests/examples/comments.beancount"),
        )]
        input: &str,
    ) {
        if let Err(err) = parse(input) {
            panic!("{err}");
        }
    }

    #[rstest]
    fn comments(
        #[values(
            "",
            "\n",
            "2016 - 11 - 28 close Liabilities:CreditCard:CapitalOne",
            "Hello world",
            "* Banking",
            "** Bank of America",
            ";; Transactions follow …",
            "; foo bar"
        )]
        input: &str,
    ) {
        let count = parse(input).expect("successful parse").count();
        assert_eq!(count, 0);
    }

    #[rstest]
    #[case("2016-11-28 close Liabilities:CreditCard:CapitalOne", account::Type::Liabilities, &["CreditCard", "CapitalOne"])]
    #[case("2016-11-28  close\tLiabilities:CreditCard:CapitalOne", account::Type::Liabilities, &["CreditCard", "CapitalOne"])]
    fn close_directive(
        #[case] input: &str,
        #[case] expected_account_type: account::Type,
        #[case] expected_account_components: &[&str],
    ) {
        let directive = parse_single_directive(input);
        let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
        assert_eq!(close.date(), Date::new(2016, 11, 28));
        assert_eq!(close.account().type_(), expected_account_type);
        assert_eq!(close.account().components(), expected_account_components);
    }

    fn parse_single_directive(input: &str) -> Directive<'_> {
        let mut iter = match parse(input) {
            Ok(iter) => iter,
            Err(err) => panic!("{err}"),
        };
        let directive = iter.next().expect("There was no directives");
        assert!(iter.next().is_none(), "There was more than one directive");
        directive
    }
}
