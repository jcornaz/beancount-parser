use std::collections::{HashMap, HashSet};

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
/// let beancount = beancount_parser_2::parse::<f64>(input).unwrap();
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
pub struct Transaction<'a, D> {
    /// Transaction flag (`*` or `!` or `None` when using the `txn` keyword)
    pub flag: Option<Flag>,
    /// Payee (if present)
    pub payee: Option<&'a str>,
    /// Narration (if present)
    pub narration: Option<&'a str>,
    /// Set of tags
    pub tags: HashSet<&'a str>,
    /// Set of links
    pub links: HashSet<&'a str>,
    /// Postings
    pub postings: Vec<Posting<'a, D>>,
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
/// let beancount = beancount_parser_2::parse::<f64>(input).unwrap();
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
pub struct Posting<'a, D> {
    /// Transaction flag (`*` or `!` or `None` when absent)
    pub flag: Option<Flag>,
    /// Account modified by the posting
    pub account: Account<'a>,
    /// Amount being added to the account
    pub amount: Option<Amount<'a, D>>,
    /// Cost (content within `{` and `}`)
    pub cost: Option<Cost<'a, D>>,
    /// Price (`@` or `@@`) syntax
    pub price: Option<PostingPrice<'a, D>>,
}

/// Cost of a posting
///
/// It is the amount within `{` and `}`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Cost<'a, D> {
    /// Cost basis of the posting
    pub amount: Option<Amount<'a, D>>,
    /// The date of this cost basis
    pub date: Option<Date>,
}

/// Price of a posting
///
/// It is the amount following the `@` or `@@` symbols
#[derive(Debug, Clone)]
pub enum PostingPrice<'a, D> {
    /// Unit cost (`@`)
    Unit(Amount<'a, D>),
    /// Total cost (`@@`)
    Total(Amount<'a, D>),
}

/// Enum representing the flag (`*` or `!`) of a transaction or posting
///
/// # Example
/// ```
/// # use beancount_parser_2::{DirectiveContent, Flag};
/// let input = "2022-05-22 * \"A transaction\"";
/// let beancount = beancount_parser_2::parse::<f64>(input).unwrap();
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
pub(crate) fn parse<D: Decimal>(
    input: Span<'_>,
) -> IResult<'_, (Transaction<'_, D>, HashMap<&str, metadata::Value<'_, D>>)> {
    let (input, flag) = alt((map(flag, Some), value(None, tag("txn"))))(input)?;
    cut(do_parse(flag))(input)
}

fn flag(input: Span<'_>) -> IResult<'_, Flag> {
    alt((
        value(Flag::Completed, char('*')),
        value(Flag::Incomplete, char('!')),
    ))(input)
}

fn do_parse<D: Decimal>(
    flag: Option<Flag>,
) -> impl Fn(Span<'_>) -> IResult<'_, (Transaction<'_, D>, HashMap<&str, metadata::Value<'_, D>>)> {
    move |input| {
        let (input, payee_and_narration) = opt(preceded(space1, payee_and_narration))(input)?;
        let (input, (tags, links)) = tags_and_links(input)?;
        let (input, _) = end_of_line(input)?;
        let (input, metadata) = metadata::parse(input)?;
        let (input, postings) = many0(posting)(input)?;
        Ok((
            input,
            (
                Transaction {
                    flag,
                    payee: payee_and_narration.and_then(|(p, _)| p),
                    narration: payee_and_narration.map(|(_, n)| n),
                    tags,
                    links,
                    postings,
                },
                metadata,
            ),
        ))
    }
}

pub(super) enum TagOrLink<'a> {
    Tag(&'a str),
    Link(&'a str),
}

pub(super) fn parse_tag(input: Span<'_>) -> IResult<'_, &str> {
    map(
        preceded(
            char('#'),
            take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
        ),
        |s: Span<'_>| *s.fragment(),
    )(input)
}

pub(super) fn parse_link(input: Span<'_>) -> IResult<'_, &str> {
    map(
        preceded(
            char('^'),
            take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
        ),
        |s: Span<'_>| *s.fragment(),
    )(input)
}

pub(super) fn parse_tag_or_link(input: Span<'_>) -> IResult<'_, TagOrLink<'_>> {
    alt((
        map(parse_tag, TagOrLink::Tag),
        map(parse_link, TagOrLink::Link),
    ))(input)
}

fn tags_and_links(input: Span<'_>) -> IResult<'_, (HashSet<&str>, HashSet<&str>)> {
    let mut tags_and_links_iter = iterator(input, preceded(space1, parse_tag_or_link));
    let (tags, links) = tags_and_links_iter.fold(
        (HashSet::new(), HashSet::new()),
        |(mut tags, mut links), x| {
            match x {
                TagOrLink::Tag(tag) => tags.insert(tag),
                TagOrLink::Link(link) => links.insert(link),
            };
            (tags, links)
        },
    );
    let (input, _) = tags_and_links_iter.finish()?;
    Ok((input, (tags, links)))
}

fn payee_and_narration(input: Span<'_>) -> IResult<'_, (Option<&str>, &str)> {
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

fn posting<D: Decimal>(input: Span<'_>) -> IResult<'_, Posting<'_, D>> {
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

fn cost<D: Decimal>(input: Span<'_>) -> IResult<'_, Cost<'_, D>> {
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
