use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    combinator::{cut, map, opt, value},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
};

use crate::{
    account::{self, Account},
    amount::{self, Amount},
    end_of_line, metadata, string, IResult, Span,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct Transaction<'a> {
    pub flag: Option<Flag>,
    pub payee: Option<&'a str>,
    pub narration: Option<&'a str>,
    pub postings: Vec<Posting<'a>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Posting<'a> {
    pub flag: Option<Flag>,
    pub account: Account<'a>,
    pub amount: Option<Amount<'a>>,
    pub lot: Option<Lot<'a>>,
    pub price: Option<Amount<'a>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Lot<'a> {
    pub cost: Option<Amount<'a>>,
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

pub(crate) fn parse(
    input: Span<'_>,
) -> IResult<'_, (Transaction<'_>, HashMap<&str, metadata::Value<'_>>)> {
    let (input, flag) = alt((map(flag, Some), value(None, tag("txn"))))(input)?;
    cut(do_parse(flag))(input)
}

fn flag(input: Span<'_>) -> IResult<'_, Flag> {
    alt((
        value(Flag::Completed, char('*')),
        value(Flag::Incomplete, char('!')),
    ))(input)
}

fn do_parse(
    flag: Option<Flag>,
) -> impl Fn(Span<'_>) -> IResult<'_, (Transaction<'_>, HashMap<&str, metadata::Value<'_>>)> {
    move |input| {
        let (input, payee_and_narration) = opt(preceded(space1, payee_and_narration))(input)?;
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
                    postings,
                },
                metadata,
            ),
        ))
    }
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

fn posting(input: Span<'_>) -> IResult<'_, Posting<'_>> {
    let (input, _) = space1(input)?;
    let (input, flag) = opt(terminated(flag, space1))(input)?;
    let (input, account) = account::parse(input)?;
    let (input, amounts) = opt(tuple((
        preceded(space1, amount::parse),
        opt(preceded(space1, lot)),
        opt(preceded(
            delimited(space1, char('@'), space1),
            amount::parse,
        )),
    )))(input)?;
    let (input, _) = end_of_line(input)?;
    let (amount, lot, price) = match amounts {
        Some((a, l, p)) => (Some(a), l, p),
        None => (None, None, None),
    };
    Ok((
        input,
        Posting {
            flag,
            account,
            amount,
            price,
            lot,
        },
    ))
}

fn lot(input: Span<'_>) -> IResult<'_, Lot<'_>> {
    let (input, _) = terminated(char('{'), space0)(input)?;
    let (input, cost) = amount::parse(input)?;
    let (input, _) = preceded(space0, char('}'))(input)?;
    Ok((input, Lot { cost: Some(cost) }))
}
