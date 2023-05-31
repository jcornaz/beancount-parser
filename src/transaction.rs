use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, space0, space1},
    combinator::{cut, iterator, map, opt, success, value},
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

use crate::{
    account::{self, Account},
    amount::{self, Amount},
    date, end_of_line, metadata, string, Date, Decimal, IResult, Span,
};

/// A transaction
///
/// It notably contains a list of [`Posting`]
///
/// # Example
/// ```
/// # use beancount_parser_2::{DirectiveContent, Flag};
/// let input = r#"
/// 2022-05-22 * "Grocery store" "Grocery shopping" #food
///   Assets:Cash           -10 CHF
///   Expenses:Groceries
/// "#;
///
/// let beancount = beancount_parser_2::parse::<&str, f64>(input).unwrap();
/// let DirectiveContent::Transaction(trx) = &beancount.directives[0].content else {
///   unreachable!("was not a transaction")
/// };
/// assert_eq!(trx.flag, Some(Flag::Completed));
/// assert_eq!(trx.payee, Some("Grocery store"));
/// assert_eq!(trx.narration, Some("Grocery shopping"));
/// assert!(trx.tags.contains("food"));
/// assert_eq!(trx.postings.len(), 2);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Transaction<S, D> {
    /// Transaction flag (`*` or `!` or `None` when using the `txn` keyword)
    pub flag: Option<Flag>,
    /// Payee (if present)
    pub payee: Option<S>,
    /// Narration (if present)
    pub narration: Option<S>,
    /// Set of tags
    pub tags: HashSet<S>,
    /// Postings
    pub postings: Vec<Posting<S, D>>,
}

/// A transaction posting
///
/// # Example
/// ```
/// # use beancount_parser_2::{DirectiveContent, Flag, PostingPrice};
/// let input = r#"
/// 2022-05-22 * "Grocery shopping"
///   Assets:Cash           1 CHF {2 PLN} @ 3 EUR
///   Expenses:Groceries
/// "#;
///
/// let beancount = beancount_parser_2::parse::<&str, f64>(input).unwrap();
/// let DirectiveContent::Transaction(trx) = &beancount.directives[0].content else {
///   unreachable!("was not a transaction")
/// };
/// let posting = &trx.postings[0];
/// assert_eq!(posting.account.as_str(), "Assets:Cash");
/// assert_eq!(posting.amount.as_ref().unwrap().value, 1.0);
/// assert_eq!(posting.amount.as_ref().unwrap().currency.as_str(), "CHF");
/// assert_eq!(posting.cost.as_ref().unwrap().amount.unwrap().value, 2.0);
/// assert_eq!(posting.cost.as_ref().unwrap().amount.unwrap().currency.as_str(), "PLN");
/// let Some(PostingPrice::Unit(price)) = &posting.price else {
///   unreachable!("no price");
/// };
/// assert_eq!(price.value, 3.0);
/// assert_eq!(price.currency.as_str(), "EUR");
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Posting<S, D> {
    /// Transaction flag (`*` or `!` or `None` when absent)
    pub flag: Option<Flag>,
    /// Account modified by the posting
    pub account: Account<S>,
    /// Amount being added to the account
    pub amount: Option<Amount<S, D>>,
    /// Cost (`@` or `@@`) syntax
    pub cost: Option<Cost<S, D>>,
    /// Price (content within `{` and `}`)
    pub price: Option<PostingPrice<S, D>>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Cost<S, D> {
    pub amount: Option<Amount<S, D>>,
    pub date: Option<Date>,
}

/// Price of a posting
///
/// It is the amount following the `@` or `@@` symbols
#[derive(Debug, Clone)]
pub enum PostingPrice<S, D> {
    /// Unit cost (`@`)
    Unit(Amount<S, D>),
    /// Total cost (`@@`)
    Total(Amount<S, D>),
}

/// Enum representing the flag (`*` or `!`) of a transaction or posting
///
/// # Example
/// ```
/// # use beancount_parser_2::{DirectiveContent, Flag};
/// let input = "2022-05-22 * \"A transaction\"";
/// let beancount = beancount_parser_2::parse::<&str, f64>(input).unwrap();
/// let DirectiveContent::Transaction(trx) = &beancount.directives[0].content else {
///   unreachable!("was not a transaction")
/// };
/// assert_eq!(trx.flag, Some(Flag::Completed));
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Flag {
    /// Completed (the char '*')
    #[default]
    Completed,
    /// Incomplete (the char '!')
    Incomplete,
}

impl From<Flag> for char {
    fn from(value: Flag) -> Self {
        match value {
            Flag::Completed => '*',
            Flag::Incomplete => '!',
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn parse<'a, S: From<&'a str> + Eq + Hash, D: Decimal>(
    input: Span<'a>,
) -> IResult<'a, (Transaction<S, D>, HashMap<S, metadata::Value<S, D>>)> {
    let (input, flag) = alt((map(flag, Some), value(None, tag("txn"))))(input)?;
    cut(do_parse(flag))(input)
}

fn flag(input: Span<'_>) -> IResult<'_, Flag> {
    alt((
        value(Flag::Completed, char('*')),
        value(Flag::Incomplete, char('!')),
    ))(input)
}

fn do_parse<'a, S: From<&'a str> + Eq + Hash, D: Decimal>(
    flag: Option<Flag>,
) -> impl Fn(Span<'a>) -> IResult<'a, (Transaction<S, D>, HashMap<S, metadata::Value<S, D>>)> {
    move |input| {
        let (input, payee_and_narration) = opt(preceded(space1, payee_and_narration))(input)?;
        let (input, tags) = tags(input)?;
        let (input, _) = end_of_line(input)?;
        let (input, metadata) = metadata::parse(input)?;
        let (input, postings) = many0(posting)(input)?;
        let (payee, narration) = match payee_and_narration {
            Some((payee, narration)) => (payee, Some(narration)),
            None => (None, None),
        };
        Ok((
            input,
            (
                Transaction {
                    flag,
                    payee,
                    narration,
                    tags,
                    postings,
                },
                metadata,
            ),
        ))
    }
}

pub(super) fn parse_tag<'a, S: From<&'a str>>(input: Span<'a>) -> IResult<'a, S> {
    map(
        preceded(
            char('#'),
            take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
        ),
        |s: Span<'_>| (*s.fragment()).into(),
    )(input)
}

fn tags<'a, S: From<&'a str> + Eq + Hash>(input: Span<'a>) -> IResult<'a, HashSet<S>> {
    let mut tags_iter = iterator(input, preceded(space1, parse_tag));
    let tags = tags_iter.collect();
    let (input, _) = tags_iter.finish()?;
    Ok((input, tags))
}

fn payee_and_narration<'a, S: From<&'a str>>(input: Span<'a>) -> IResult<'a, (Option<S>, S)> {
    let (input, s1) = string(input)?;
    let (input, s2) = opt(preceded(space1, string))(input)?;
    Ok((
        input,
        match s2 {
            Some(narration) => (Some(s1), narration),
            None => (None, s1),
        },
    ))
}

fn posting<'a, S: From<&'a str>, D: Decimal>(input: Span<'a>) -> IResult<'a, Posting<S, D>> {
    let (input, _) = space1(input)?;
    let (input, flag) = opt(terminated(flag, space1))(input)?;
    let (input, account) = account::parse(input)?;
    let (input, amounts) = opt(tuple((
        preceded(space1, amount::parse),
        opt(preceded(space1, cost)),
        opt(preceded(
            space1,
            alt((
                map(
                    preceded(tuple((char('@'), space1)), amount::parse),
                    PostingPrice::Unit,
                ),
                map(
                    preceded(tuple((tag("@@"), space1)), amount::parse),
                    PostingPrice::Total,
                ),
            )),
        )),
    )))(input)?;
    let (input, _) = end_of_line(input)?;
    let (amount, cost, price) = match amounts {
        Some((a, l, p)) => (Some(a), l, p),
        None => (None, None, None),
    };
    Ok((
        input,
        Posting {
            flag,
            account,
            amount,
            cost,
            price,
        },
    ))
}

fn cost<'a, S: From<&'a str>, D: Decimal>(input: Span<'a>) -> IResult<'a, Cost<S, D>> {
    let (input, _) = terminated(char('{'), space0)(input)?;
    let (input, (cost, date)) = alt((
        map(
            separated_pair(
                amount::parse,
                delimited(space0, char(','), space0),
                date::parse,
            ),
            |(a, d)| (Some(a), Some(d)),
        ),
        map(
            separated_pair(
                date::parse,
                delimited(space0, char(','), space0),
                amount::parse,
            ),
            |(d, a)| (Some(a), Some(d)),
        ),
        map(amount::parse, |a| (Some(a), None)),
        map(date::parse, |d| (None, Some(d))),
        map(success(true), |_| (None, None)),
    ))(input)?;
    let (input, _) = preceded(space0, char('}'))(input)?;
    Ok((input, Cost { amount: cost, date }))
}
