use nom::{
    bytes::complete::tag,
    character::complete::{char, space0, space1},
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
    postings: Vec<Posting<'a>>,
}

impl<'a> Transaction<'a> {
    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn postings(&self) -> &Vec<Posting<'a>> {
        &self.postings
    }
}

fn transaction(input: &str) -> IResult<&str, Transaction<'_>> {
    let flag = tag("*");
    map(
        tuple((
            delimited(tuple((flag, space1)), string, tuple((space0, char('\n')))),
            many1(posting),
        )),
        |(description, postings)| Transaction {
            description,
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
    }

    #[test]
    fn invalid_transaction() {
        let input = "open Assets:US:BofA:Checking";
        assert!(transaction(input).is_err());
    }
}
