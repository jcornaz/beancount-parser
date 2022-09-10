use std::str;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, space0, space1},
    combinator::{eof, map, opt},
    multi::many0,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::string::{comment, string};

mod posting;

use posting::{posting, Posting};

pub struct Transaction<'a> {
    flag: Option<Flag>,
    payee: Option<String>,
    narration: Option<String>,
    postings: Vec<Posting<'a>>,
    comment: Option<&'a str>,
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

    pub fn comment(&self) -> Option<&str> {
        self.comment
    }
}

fn transaction(input: &str) -> IResult<&str, Transaction<'_>> {
    let payee_and_narration = alt((
        separated_pair(map(string, Some), space1, string),
        map(string, |n| (None, n)),
    ));
    map(
        terminated(
            tuple((
                alt((map(tag("txn"), |_| None), map(flag, Some))),
                opt(preceded(space1, payee_and_narration)),
                opt(preceded(space0, comment)),
                many0(preceded(tuple((line_ending, space1)), posting)),
            )),
            alt((line_ending, eof)),
        ),
        |(flag, payee_and_narration, comment, postings)| {
            let (payee, narration) = match payee_and_narration {
                Some((p, n)) => (p, Some(n)),
                None => (None, None),
            };
            Transaction {
                flag,
                payee,
                narration,
                postings,
                comment,
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
    use rstest::rstest;

    #[rstest]
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
        assert!(transaction.comment().is_none());
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
    fn transaction_with_comment() {
        let input = r#"txn "Hello \"world\"" ; And a comment!"#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert_eq!(transaction.comment(), Some("And a comment!"));
    }

    #[rstest]
    fn invalid_transaction(
        #[values(
            "open Assets:US:BofA:Checking",
            r#"*"hello""#,
            r#"* "hello" Assets:A 10 CHF"#
        )]
        input: &str,
    ) {
        assert!(transaction(input).is_err());
    }
}
