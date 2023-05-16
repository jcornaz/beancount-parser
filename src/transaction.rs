use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space1},
    combinator::{cut, map, opt, value},
    multi::many0,
    sequence::{preceded, terminated},
};

use crate::{
    account::{self, Account},
    end_of_line, string, IResult, Span,
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

pub(crate) fn parse(input: Span<'_>) -> IResult<'_, Transaction<'_>> {
    let (input, flag) = alt((map(flag, Some), value(None, tag("txn"))))(input)?;
    cut(do_parse(flag))(input)
}

fn flag(input: Span<'_>) -> IResult<'_, Flag> {
    alt((
        value(Flag::Completed, char('*')),
        value(Flag::Incomplete, char('!')),
    ))(input)
}

fn do_parse(flag: Option<Flag>) -> impl Fn(Span<'_>) -> IResult<'_, Transaction<'_>> {
    move |input| {
        let (input, payee_and_narration) = opt(preceded(space1, payee_and_narration))(input)?;
        let (input, postings) = many0(preceded(end_of_line, posting))(input)?;
        Ok((
            input,
            Transaction {
                flag,
                payee: payee_and_narration.and_then(|(p, _)| p),
                narration: payee_and_narration.map(|(_, n)| n),
                postings,
            },
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
    Ok((input, Posting { flag, account }))
}
