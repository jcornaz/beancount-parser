#![cfg(test)]

use crate::{account, Account, Close, Date, Directive};
use pest::Parser as Parse;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "beancount.pest"]
struct Parser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn parse(input: &str) -> Result<impl Iterator<Item = Directive<'_>>, Box<dyn std::error::Error>> {
    Ok(Parser::parse(Rule::beancount, input)?
        .flatten()
        .filter_map(|p| match p.as_rule() {
            Rule::close_directive => Some(Directive::Close(build_close_directive(p))),
            _ => None,
        }))
}

fn build_close_directive(pair: Pair<'_>) -> Close<'_> {
    let mut date = Date::new(0, 0, 0);
    let mut account = None;
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::date => date = build_date(p),
            Rule::account => account = Some(build_account(p)),
            _ => (),
        }
    }
    Close {
        date,
        account: account.expect("account not found"),
    }
}

fn build_date(pair: Pair<'_>) -> Date {
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::year => year = pair.as_str().parse().expect("invalid year"),
            Rule::month => month = pair.as_str().parse().expect("invalid year"),
            Rule::day => day = pair.as_str().parse().expect("invalid year"),
            _ => (),
        }
    }
    Date::new(year, month, day)
}

fn build_account(pair: Pair<'_>) -> Account<'_> {
    let mut type_ = None;
    let mut components = Vec::new();
    for comp in pair.into_inner() {
        match (comp.as_rule(), comp.as_str()) {
            (Rule::account_type, "Liabilities") => type_ = Some(account::Type::Liabilities),
            (Rule::account_component, name) => components.push(name),
            _ => (),
        }
    }
    Account {
        type_: type_.expect("account type not found"),
        components,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_beancount() {
        let directive_count = parse("").expect("should parse successfully").count();
        assert_eq!(directive_count, 0);
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
        let mut iter = parse(input).expect("Parse failed");
        let directive = iter.next().expect("There was no directives");
        assert!(iter.next().is_none(), "There was more than one directive");
        directive
    }
}
