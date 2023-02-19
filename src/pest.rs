#![cfg(all(test, feature = "unstable"))]

use pest::Parser as Parse;
use pest_derive::Parser;

use std::collections::HashMap;

use crate::transaction::{Flag, Info};
use crate::{account, Account, Close, Date, Directive, Open, Transaction};

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
            Rule::transaction => Some(Directive::Transaction(build_transaction(p))),
            Rule::open => Some(Directive::Open(build_open_directive(p))),
            Rule::close => Some(Directive::Close(build_close_directive(p))),
            _ => None,
        }))
}

fn build_transaction(pair: Pair<'_>) -> Transaction<'_> {
    let mut inner = pair.into_inner();
    let date = build_date(inner.next().expect("no date in transaction"));
    let mut flag = None;
    let mut payee = None;
    let mut narration = None;
    for pair in inner {
        match (pair.as_rule(), pair.as_str(), narration.take()) {
            (Rule::transaction_flag, "*", _) => flag = Some(Flag::Cleared),
            (Rule::transaction_flag, "!", _) => flag = Some(Flag::Pending),
            (Rule::string, s, None) => narration = Some(s.into()),
            (Rule::string, s, Some(n)) => {
                payee = Some(n);
                narration = Some(s.into());
            }
            _ => (),
        }
    }
    Transaction {
        info: Info {
            date,
            flag,
            payee,
            narration,
            tags: vec![],
            comment: None,
        },
        metadata: HashMap::default(),
        postings: vec![],
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Flag;

    #[rstest]
    fn successful_parse(
        #[values("", " ", " \n ", " \t ",
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
    #[case(
        "2016-11-28 close Liabilities:CreditCard:CapitalOne",
        Date::new(2016, 11, 28)
    )]
    #[case("2022-12-31 open Assets:A", Date::new(2022, 12, 31))]
    #[case("2000-01-01 txn", Date::new(2000, 1, 1))]
    #[case("2000-01-02 * \"Groceries\"", Date::new(2000, 1, 2))]
    #[case("2000-01-03 * \"Store\" \"Groceries\"", Date::new(2000, 1, 3))]
    #[case("2000-01-04 *", Date::new(2000, 1, 4))]
    #[case("2000-01-05 !", Date::new(2000, 1, 5))]
    fn parse_date(#[case] input: &str, #[case] expected: Date) {
        let directive = parse_single_directive(input);
        assert_eq!(directive.date(), Some(expected));
    }

    #[rstest]
    #[case("2000-01-01 txn", None)]
    #[case("2000-01-01 txn \"Store\"", None)]
    #[case("2000-01-01 *", Some(Flag::Cleared))]
    #[case("2000-01-01 * \"Store\"", Some(Flag::Cleared))]
    #[case("2000-01-01 !", Some(Flag::Pending))]
    #[case("2000-01-01 ! \"Store\"", Some(Flag::Pending))]
    fn parse_transaction_flag(#[case] input: &str, #[case] expected: Option<Flag>) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        assert_eq!(transaction.flag(), expected);
    }

    #[rstest]
    #[case("2022-02-12 txn", None, None)]
    #[case("2022-02-12 *", None, None)]
    #[case("2022-02-12 txn \"Hello\"", None, Some("Hello"))]
    #[case("2022-02-12 * \"Hello\"", None, Some("Hello"))]
    #[case("2022-02-12 txn \"Hello\" \"World\"", Some("Hello"), Some("World"))]
    #[case("2022-02-12 ! \"Hello\" \"World\"", Some("Hello"), Some("World"))]
    fn parse_transaction_payee_and_description(
        #[case] input: &str,
        #[case] expected_payee: Option<&str>,
        #[case] expected_narration: Option<&str>,
    ) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        assert_eq!(transaction.payee(), expected_payee);
        assert_eq!(transaction.narration(), expected_narration);
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
        assert_eq!(close.account().type_(), expected_account_type);
    }

    #[rstest]
    #[case("2016-11-28 open Assets:Hello", account::Type::Assets)]
    #[case("2016-11-28 open Liabilities:Hello", account::Type::Liabilities)]
    fn parse_open_directive_account_type(
        #[case] input: &str,
        #[case] expected_account_type: account::Type,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
        assert_eq!(open.account().type_(), expected_account_type);
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
        assert_eq!(close.account().components(), expected_account_components);
    }

    #[rstest]
    #[case("2016-11-28 open Liabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    #[case("2016-11-28 open Assets:Hello", &["Hello"])]
    #[case("2016-11-28 open Assets", &[])]
    #[case("2016-11-28 open Assets:Hello-World:123", &["Hello-World", "123"])]
    #[case("2016-11-28  open\t\tLiabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    fn parse_open_directive_account_components(
        #[case] input: &str,
        #[case] expected_account_components: &[&str],
    ) {
        let directive = parse_single_directive(input);
        let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
        assert_eq!(open.account().components(), expected_account_components);
    }

    #[rstest]
    #[case("2016-11-28 open Assets", &[])]
    #[case("2016-11-28 open Assets CHF", &["CHF"])]
    #[case("2016-11-28 open Assets CHF,EUR", &["CHF", "EUR"])]
    #[case("2016-11-28 open Assets CHF , EUR", &["CHF", "EUR"])]
    #[case("2016-11-28 open Assets AB-CD, A_2B, A.B, A'B", &["AB-CD", "A_2B", "A.B", "A'B"])]
    fn parse_open_directive_currencies(#[case] input: &str, #[case] expected: &[&str]) {
        let directive = parse_single_directive(input);
        let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
        assert_eq!(open.currencies(), expected);
    }

    #[rstest]
    fn error_case(
        #[values(
            "2016-11-28closeLiabilities:CreditCard:CapitalOne",
            "2016-11-28 closeLiabilities:CreditCard:CapitalOne",
            "2016-11-28close Liabilities:CreditCard:CapitalOne",
            "2016-11-28 close Liabilities:CreditCard:CapitalOne Oops",
            "2016-11-28 close Oops",
            "2016-11-28openAssets:A",
            "2016-11-28 openAssets:A",
            "2016-11-28open Assets:A",
            "2016-11-28 open Assets:A oops",
            "2016-11-28 open Assets:A 22",
            "2016-11-28 open Oops"
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
