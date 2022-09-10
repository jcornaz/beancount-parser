use std::str;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{map, opt},
    multi::many0,
    sequence::{preceded, separated_pair, tuple},
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
    let payee_and_narration = alt((
        separated_pair(map(string, Some), separator, string),
        map(string, |n| (None, n)),
    ));
    map(
        tuple((
            alt((map(tag("txn"), |_| None), map(flag, Some))),
            opt(preceded(separator, payee_and_narration)),
            many0(preceded(separator, posting)),
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

fn separator(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_whitespace())(input)
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
    use rstest::rstest;

    #[rstest]
    fn simple_transaction(
        #[values(
            r#"* "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#,
            r#"* "Hello \"world\"" Expenses:A 10 CHF Assets:B -10 CHF"#
        )]
        input: &str,
    ) {
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert_eq!(transaction.narration(), Some(r#"Hello "world""#));
        assert_eq!(transaction.postings().len(), 2);
        assert_eq!(transaction.flag(), Some(Flag::Cleared));
        assert!(transaction.payee().is_none());
    }

    #[test]
    fn transaction_without_posting() {
        let input = r#"* "Hello \"world\"""#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert!(transaction.postings().is_empty());
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
    fn without_space_after_flag() {
        let input = r#"*"hello""#;
        let (rest, _) = transaction(input).unwrap();
        assert_eq!(rest, r#""hello""#);
    }

    #[test]
    fn invalid_transaction() {
        let input = "open Assets:US:BofA:Checking";
        assert!(transaction(input).is_err());
    }
}
