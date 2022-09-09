use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::{map, opt},
    multi::many1,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

use crate::string::string;

mod posting;

use posting::{posting, Posting};

pub struct Transaction<'a> {
    flag: Option<Flag>,
    payee: Option<String>,
    narration: Option<String>,
    postings: Vec<Posting<'a>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Flag {
    Cleared,
    Pending,
}

impl<'a> Transaction<'a> {
    pub fn payee(&self) -> Option<&str> {
        self.payee.as_deref()
    }

    pub fn narration(&self) -> Option<&str> {
        self.narration.as_deref()
    }

    pub fn postings(&self) -> &Vec<Posting<'a>> {
        &self.postings
    }

    pub fn flag(&self) -> Option<Flag> {
        self.flag
    }
}

fn transaction(input: &str) -> IResult<&str, Transaction<'_>> {
    let payee_and_narration = opt(alt((
        separated_pair(map(string, Some), space0, string),
        map(string, |n| (None, n)),
    )));
    map(
        tuple((
            alt((map(tag("txn"), |_| None), map(flag, Some))),
            delimited(space0, payee_and_narration, tuple((space0, char('\n')))),
            many1(posting),
        )),
        |(flag, payee_and_narration, postings)| {
            let (payee, narration) = match payee_and_narration {
                Some((p, n)) => (p, Some(n)),
                None => (None, None),
            };
            Transaction {
                flag,
                payee,
                narration,
                postings,
            }
        },
    )(input)
}

fn flag(input: &str) -> IResult<&str, Flag> {
    alt((
        map(char('*'), |_| Flag::Cleared),
        map(char('!'), |_| Flag::Pending),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_transaction() {
        let input = r#"* "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert_eq!(transaction.narration(), Some(r#"Hello "world""#));
        assert_eq!(transaction.postings().len(), 2);
        assert_eq!(transaction.flag(), Some(Flag::Cleared));
        assert!(transaction.payee().is_none());
    }

    #[test]
    fn transaction_without_description() {
        let input = r#"*
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert!(transaction.narration().is_none());
    }

    #[test]
    fn transaction_with_payee() {
        let input = r#"* "me" "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert_eq!(transaction.payee(), Some("me"));
    }

    #[test]
    fn transaction_with_exclamation_mark() {
        let input = r#"! "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert_eq!(transaction.flag(), Some(Flag::Pending));
    }

    #[test]
    fn transaction_without_flag() {
        let input = r#"txn "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert!(transaction.flag().is_none());
    }

    #[test]
    fn invalid_transaction() {
        let input = "open Assets:US:BofA:Checking";
        assert!(transaction(input).is_err());
    }
}
