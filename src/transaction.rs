#![allow(missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
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
    date, end_of_line, metadata, string, Date, IResult, Span,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct Transaction<'a, D> {
    pub flag: Option<Flag>,
    pub payee: Option<&'a str>,
    pub narration: Option<&'a str>,
    pub tags: HashSet<&'a str>,
    pub postings: Vec<Posting<'a, D>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Posting<'a, D> {
    pub flag: Option<Flag>,
    pub account: Account<'a>,
    pub amount: Option<Amount<'a, D>>,
    pub cost: Option<Cost<'a, D>>,
    pub price: Option<Amount<'a, D>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Cost<'a, D> {
    pub amount: Option<Amount<'a, D>>,
    pub date: Option<Date>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Flag {
    #[default]
    Completed,
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

pub(crate) fn parse<D: FromStr>(
    input: Span<'_>,
) -> IResult<'_, (Transaction<'_, D>, HashMap<&str, metadata::Value<'_>>)> {
    let (input, flag) = alt((map(flag, Some), value(None, tag("txn"))))(input)?;
    cut(do_parse(flag))(input)
}

fn flag(input: Span<'_>) -> IResult<'_, Flag> {
    alt((
        value(Flag::Completed, char('*')),
        value(Flag::Incomplete, char('!')),
    ))(input)
}

fn do_parse<D: FromStr>(
    flag: Option<Flag>,
) -> impl Fn(Span<'_>) -> IResult<'_, (Transaction<'_, D>, HashMap<&str, metadata::Value<'_>>)> {
    move |input| {
        let (input, payee_and_narration) = opt(preceded(space1, payee_and_narration))(input)?;
        let (input, tags) = tags(input)?;
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
                    postings,
                },
                metadata,
            ),
        ))
    }
}

fn tags(input: Span<'_>) -> IResult<'_, HashSet<&str>> {
    let mut tags_iter = iterator(
        input,
        preceded(
            space1,
            preceded(
                char('#'),
                take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
            ),
        ),
    );
    let tags = tags_iter.map(|s: Span<'_>| *s.fragment()).collect();
    let (input, _) = tags_iter.finish()?;
    Ok((input, tags))
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

fn posting<D: FromStr>(input: Span<'_>) -> IResult<'_, Posting<'_, D>> {
    let (input, _) = space1(input)?;
    let (input, flag) = opt(terminated(flag, space1))(input)?;
    let (input, account) = account::parse(input)?;
    let (input, amounts) = opt(tuple((
        preceded(space1, amount::parse),
        opt(preceded(space1, cost)),
        opt(preceded(
            delimited(space1, char('@'), space1),
            amount::parse,
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

fn cost<D: FromStr>(input: Span<'_>) -> IResult<'_, Cost<'_, D>> {
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
