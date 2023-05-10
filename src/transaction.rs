//! Types for representing an [`Transaction`]

#[cfg(feature = "unstable")]
use std::collections::HashMap;
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

use posting::posting;
pub use posting::{Posting, PriceType};

#[cfg(feature = "unstable")]
use crate::metadata::Metadata;
use crate::{
    date::date,
    string::{comment, string},
    Date,
};
#[cfg(feature = "unstable")]
use crate::{
    pest_parser::{Pair, Rule},
    string,
};

pub(crate) mod posting;

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
#[derive(Debug, Clone)]
pub struct Transaction<'a> {
    date: Date,
    flag: Option<Flag>,
    payee: Option<String>,
    narration: Option<String>,
    tags: Vec<&'a str>,
    comment: Option<&'a str>,
    #[cfg(feature = "unstable")]
    metadata: Metadata<'a>,
    postings: Vec<Posting<'a>>,
}

/// The transaction flag
///
/// It is eithe cleared (`*`) of pending (`!`)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum Flag {
    /// Cleared flag (the `*` character)
    #[default]
    Cleared,
    /// Pending flag (the `!` character)
    Pending,
}

impl Flag {
    #[cfg(feature = "unstable")]
    fn from_pair(pair: Pair<'_>) -> Flag {
        match pair.as_str() {
            "*" => Flag::Cleared,
            "!" => Flag::Pending,
            _ => unreachable!("Invalid transaction flag"),
        }
    }
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

    /// Returns the metadata
    #[must_use]
    #[cfg(feature = "unstable")]
    pub fn metadata(&self) -> &Metadata<'a> {
        &self.metadata
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

    #[cfg(feature = "unstable")]
    pub(crate) fn from_pair(pair: Pair<'_>) -> Transaction<'_> {
        let mut inner = pair.into_inner();
        let date = Date::from_pair(inner.next().expect("no date in transaction"));
        let mut flag = None;
        let mut payee = None;
        let mut narration = None;
        let mut postings = Vec::new();
        let mut tags = Vec::new();
        for pair in inner {
            match pair.as_rule() {
                Rule::transaction_flag => flag = Some(Flag::from_pair(pair)),
                Rule::payee => {
                    payee = Some(
                        string::from_pair(pair.into_inner().next().expect("no string in payee"))
                            .into(),
                    );
                }
                Rule::narration => {
                    narration = Some(
                        string::from_pair(
                            pair.into_inner().next().expect("no string in narration"),
                        )
                        .into(),
                    );
                }
                Rule::postings => postings = pair.into_inner().map(Posting::from_pair).collect(),
                Rule::tags => {
                    tags = pair
                        .into_inner()
                        .filter_map(|p| p.as_str().strip_prefix('#'))
                        .collect();
                }
                _ => (),
            }
        }
        Transaction {
            date,
            flag,
            payee,
            narration,
            tags,
            comment: None,
            metadata: HashMap::default(),
            postings,
        }
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
                crate::metadata::metadata,
                many0(preceded(tuple((line_ending, space1)), posting)),
            )),
            cut(alt((line_ending, eof))),
        ),
        #[allow(unused_variables)]
        |(date, flag, payee_and_narration, tags, comment, metadata, postings)| {
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
                comment,
                #[cfg(feature = "unstable")]
                metadata,
                postings,
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
