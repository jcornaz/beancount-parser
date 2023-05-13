//! Types for representing an [`Transaction`]

#[cfg(feature = "unstable")]
use std::collections::HashMap;
use std::str;

use nom::bytes::complete::{self, take_till};

use nom::Parser;
use nom::{
    branch::alt,
    character::complete::{char, line_ending, space0, space1},
    combinator::{cut, eof, map, opt},
    multi::many0,
    sequence::{preceded, separated_pair, terminated, tuple},
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
use crate::{IResult, Span};

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Flag {
    /// Cleared flag (the `*` character)
    Cleared,
    /// Pending flag (the `!` character)
    Pending,
}

impl Default for Flag {
    fn default() -> Self {
        Self::Cleared
    }
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

pub(crate) fn transaction(input: Span<'_>) -> IResult<'_, Transaction<'_>> {
    let (input, date) = terminated(date, space1)(input)?;
    let (input, flag) = alt((map(flag, Some), map(complete::tag("txn"), |_| None)))(input)?;
    let (input, (payee, narration)) = payee_and_narration(input)?;
    let (input, tags) = many0(preceded(space0, tag))(input)?;
    let (input, _) = space0(input)?;
    let (input, comment) = opt(comment)(input)?;
    #[cfg_attr(not(feature = "unstable"), allow(unused_variables))]
    let (input, metadata) = crate::metadata::metadata(input)?;
    let (input, postings) = many0(preceded(tuple((line_ending, space1)), posting))(input)?;
    let (input, _) = cut(alt((line_ending, eof)))(input)?;
    Ok((
        input,
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
        },
    ))
}

fn payee_and_narration(input: Span<'_>) -> IResult<'_, (Option<String>, Option<String>)> {
    let (input, opt) = opt(preceded(
        space1,
        alt((
            separated_pair(string.map(Some), space1, string.map(Some)),
            string.map(|n| (None, Some(n))),
        )),
    ))(input)?;
    Ok((input, opt.unwrap_or((None, None))))
}

pub(crate) fn tag(input: Span<'_>) -> IResult<'_, &str> {
    preceded(
        char('#'),
        take_till(|c: char| c.is_whitespace() || c == '#').map(|s: Span<'_>| *s.fragment()),
    )(input)
}

fn flag(input: Span<'_>) -> IResult<'_, Flag> {
    alt((
        map(char('*'), |_| Flag::Cleared),
        map(char('!'), |_| Flag::Pending),
    ))(input)
}
