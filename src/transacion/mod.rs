use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of, space0, space1},
    combinator::map,
    multi::many1,
    sequence::{delimited, tuple},
    IResult,
};

use crate::string::string;

mod posting;

use posting::{posting, Posting};

pub struct Transaction<'a> {
    description: String,
    flag: Option<char>,
    postings: Vec<Posting<'a>>,
}

impl<'a> Transaction<'a> {
    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn postings(&self) -> &Vec<Posting<'a>> {
        &self.postings
    }

    pub fn flag(&self) -> Option<char> {
        self.flag
    }
}

fn transaction(input: &str) -> IResult<&str, Transaction<'_>> {
    let flag = alt((map(tag("txn"), |_| None), map(one_of("*!"), Some)));
    let description = delimited(space1, string, tuple((space0, char('\n'))));
    map(
        tuple((flag, description, many1(posting))),
        |(flag, description, postings)| Transaction {
            description,
            flag,
            postings,
        },
    )(input)
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
        assert_eq!(transaction.description(), r#"Hello "world""#);
        assert_eq!(transaction.postings().len(), 2);
        assert_eq!(transaction.flag(), Some('*'));
    }

    #[test]
    fn transaction_with_exclamation_mark() {
        let input = r#"! "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should succesfully parse the transaction");
        assert_eq!(transaction.flag(), Some('!'));
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
