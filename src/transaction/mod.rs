//! Types for representing an [`Transaction`]

use std::str;

use nom::bytes::complete::{self, take_till};
use nom::{
    branch::alt,
    character::complete::{char, line_ending, space0, space1},
    combinator::{cut, eof, map, opt},
    multi::many0,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    date::date,
    string::{comment, string},
    Date,
};

mod balanced;
mod posting;

use posting::posting;
pub use posting::{Posting, PriceType};

/// A transaction
///
/// Contains, a potential narration as well as the [`Posting`]s.
///
/// # Example
/// ```beancount
/// 2022-09-11 * "Coffee beans"
///   Expenses:Groceries   10 CHF
///   Assets:Bank
/// ```
#[cfg(not(feature = "unstable"))]
#[derive(Debug, Clone)]
pub struct Transaction<'a> {
    date: Date,
    flag: Option<Flag>,
    payee: Option<String>,
    narration: Option<String>,
    tags: Vec<&'a str>,
    postings: Vec<Posting<'a>>,
    comment: Option<&'a str>,
}

/// A transaction
///
/// Contains, a potential narration as well as the [`Posting`]s.
///
/// # Example
/// ```beancount
/// 2022-09-11 * "Coffee beans"
///   Expenses:Groceries   10 CHF
///   Assets:Bank
/// ```
#[cfg(feature = "unstable")]
#[derive(Debug, Clone)]
pub struct Transaction<'a, A = Option<crate::Amount<'a>>> {
    date: Date,
    flag: Option<Flag>,
    payee: Option<String>,
    narration: Option<String>,
    tags: Vec<&'a str>,
    postings: Vec<Posting<'a, A>>,
    comment: Option<&'a str>,
}

/// The transaction flag
///
/// It is eithe cleared (`*`) of pending (`!`)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Flag {
    /// Cleared flag (the `*` character)
    Cleared,
    /// Pending flag (the `!` character)
    Pending,
}

impl<'a> Transaction<'a> {
    /// Returns the "payee" if one was defined
    #[must_use]
    pub fn payee(&self) -> Option<&str> {
        self.payee.as_deref()
    }

    /// Returns the "narration" if one was defined
    #[must_use]
    pub fn narration(&self) -> Option<&str> {
        self.narration.as_deref()
    }

    /// Returns the postings
    #[must_use]
    pub fn postings(&self) -> &[Posting<'a>] {
        &self.postings
    }

    /// Returns the flag of the transaction (if present)
    #[must_use]
    pub fn flag(&self) -> Option<Flag> {
        self.flag
    }

    /// Returns the tags attached to this transaction
    #[must_use]
    pub fn tags(&self) -> &[&'a str] {
        &self.tags
    }

    /// Returns the comment (if present)
    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        self.comment
    }

    /// The date of the transaction
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    pub(crate) fn append_tags(&mut self, tags: &[&'a str]) {
        self.tags.extend(tags);
    }
}

pub(crate) fn transaction(input: &str) -> IResult<&str, Transaction<'_>> {
    let payee_and_narration = alt((
        separated_pair(map(string, Some), space1, string),
        map(string, |n| (None, n)),
    ));
    map(
        terminated(
            tuple((
                terminated(date, space1),
                alt((map(complete::tag("txn"), |_| None), map(flag, Some))),
                opt(preceded(space1, payee_and_narration)),
                many0(preceded(space0, tag)),
                opt(preceded(space0, comment)),
                many0(preceded(tuple((line_ending, space1)), posting)),
            )),
            cut(alt((line_ending, eof))),
        ),
        |(date, flag, payee_and_narration, tags, comment, postings)| {
            let (payee, narration) = match payee_and_narration {
                Some((p, n)) => (p, Some(n)),
                None => (None, None),
            };
            Transaction {
                date,
                flag,
                payee,
                narration,
                tags,
                postings,
                comment,
            }
        },
    )(input)
}

pub(crate) fn tag(input: &str) -> IResult<&str, &str> {
    preceded(
        char('#'),
        take_till(|c: char| c.is_whitespace() || c == '#'),
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
        let input = r#"2022-09-16 * "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert_eq!(transaction.date(), Date::new(2022, 9, 16));
        assert_eq!(transaction.narration(), Some(r#"Hello "world""#));
        assert_eq!(transaction.postings().len(), 2);
        assert_eq!(transaction.flag(), Some(Flag::Cleared));
        assert_eq!(transaction.payee(), None);
        assert_eq!(transaction.comment(), None);
        assert_eq!(transaction.tags().len(), 0);
    }

    #[test]
    fn transaction_without_posting() {
        let input = r#"2022-01-01 * "Hello \"world\"""#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert!(transaction.postings().is_empty());
    }

    #[test]
    fn transaction_without_description() {
        let input = r#"2022-01-01 *
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert!(transaction.narration().is_none());
    }

    #[test]
    fn transaction_with_payee() {
        let input = r#"2022-01-01 * "me" "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert_eq!(transaction.payee(), Some("me"));
    }

    #[test]
    fn transaction_with_exclamation_mark() {
        let input = r#"2022-01-01 ! "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert_eq!(transaction.flag(), Some(Flag::Pending));
    }

    #[test]
    fn transaction_without_flag() {
        let input = r#"2022-01-01 txn "Hello \"world\""
            Expenses:A    10 CHF
            Assets:B     -10 CHF
        "#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert!(transaction.flag().is_none());
    }

    #[test]
    fn transaction_with_one_tag() {
        let input = r#"2022-01-01 txn "Hello \"world\"" #hello-world"#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert_eq!(transaction.tags(), ["hello-world"]);
    }

    #[test]
    fn transaction_with_multiple_tags() {
        let input = r#"2022-01-01 txn "Hello \"world\"" #that #is #cool"#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert_eq!(transaction.tags(), ["that", "is", "cool"]);
    }

    #[test]
    fn transaction_with_comment() {
        let input = r#"2022-01-01 txn "Hello \"world\"" ; And a comment!"#;
        let (_, transaction) =
            transaction(input).expect("should successfully parse the transaction");
        assert_eq!(transaction.comment(), Some("And a comment!"));
    }

    #[rstest]
    fn errors(#[values("2022-01-01 open Assets:US:BofA:Checking")] input: &str) {
        let result = transaction(input);
        assert!(matches!(result, Err(nom::Err::Error(_))), "{result:?}");
    }

    #[rstest]
    fn failures(
        #[values(
            r#"2022-01-01 *"hello""#,
            r#"2022-01-01 * "hello" Assets:A 10 CHF"#,
            "2022-01-01 ! test"
        )]
        input: &str,
    ) {
        let result = transaction(input);
        assert!(matches!(result, Err(nom::Err::Failure(_))), "{result:?}");
    }

    #[rstest]
    fn simple_tag(#[values("#test", "#test ", "#test#")] input: &str) {
        let (_, tag) = tag(input).expect("should successfully parse the tag");
        assert_eq!(tag, "test");
    }
}
