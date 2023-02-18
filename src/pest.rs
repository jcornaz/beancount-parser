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
        .next()
        .expect("no root rule")
        .into_inner()
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
            "\n\n\r\n",
            "2016 - 11 - 28 close Liabilities:CreditCard:CapitalOne",
            "Hello world",
            "* Banking",
            "** Bank of America",
            ";; Transactions follow …",
            "; foo bar",
            ";"
        )]
        input: &str,
    ) {
        let count = parse(input)
            .expect("should successfully parse input")
            .count();
        assert_eq!(count, 0);
    }

    #[rstest]
    fn parse_close_directive_date() {
        let input = "2016-11-28 close Liabilities:CreditCard:CapitalOne";
        let directive = parse_single_directive(input);
        let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
        assert_eq!(close.date(), Date::new(2016, 11, 28));
    }

    #[rstest]
    #[case("2016-11-28 close Assets:Hello", account::Type::Assets)]
    #[case("2016-11-28 close Liabilities:Hello", account::Type::Liabilities)]
    #[case("2016-11-28 close Expenses:Hello", account::Type::Expenses)]
    #[case("2016-11-28 close Income:Hello", account::Type::Income)]
    #[case("2016-11-28 close Equity:Hello", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello ; Foo bar", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello; Foo bar", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello;Foo bar", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello;", account::Type::Equity)]
    fn parse_close_directive_account_type(
        #[case] input: &str,
        #[case] expected_account_type: account::Type,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
        assert_eq!(close.date(), Date::new(2016, 11, 28));
        assert_eq!(close.account().type_(), expected_account_type);
    }

    #[rstest]
    #[case("2016-11-28 close Liabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    #[case("2016-11-28 close Assets:Hello", &["Hello"])]
    #[case("2016-11-28 close Assets", &[])]
    #[case("2016-11-28 close Assets:Hello-World:123", &["Hello-World", "123"])]
    #[case("2016-11-28  close\tLiabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    fn parse_close_directive_account_components(
        #[case] input: &str,
        #[case] expected_account_components: &[&str],
    ) {
        let directive = parse_single_directive(input);
        let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
        assert_eq!(close.date(), Date::new(2016, 11, 28));
        assert_eq!(close.account().components(), expected_account_components);
    }

    #[rstest]
    fn error_case(
        #[values(
            "2016-11-28closeLiabilities:CreditCard:CapitalOne",
            "2016-11-28 closeLiabilities:CreditCard:CapitalOne",
            "2016-11-28close Liabilities:CreditCard:CapitalOne",
            "2016-11-28 close Liabilities:CreditCard:CapitalOne Oops",
            "2016-11-28 close Oops"
        )]
        input: &str,
    ) {
        let result = parse(input);
        assert!(result.is_err());
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
