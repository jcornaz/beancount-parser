use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::{cut, opt},
    sequence::preceded,
};

use crate::{string, IResult, Span};

#[derive(Debug)]
#[non_exhaustive]
pub struct Transaction<'a> {
    pub payee: Option<&'a str>,
    pub narration: Option<&'a str>,
}

pub(crate) fn parse(input: Span<'_>) -> IResult<'_, Transaction<'_>> {
    let (input, _) = tag("txn")(input)?;
    cut(do_parse)(input)
}

fn do_parse(input: Span<'_>) -> IResult<'_, Transaction<'_>> {
    let (input, payee_and_narration) = opt(preceded(space1, payee_and_narration))(input)?;
    Ok((
        input,
        Transaction {
            payee: payee_and_narration.and_then(|(p, _)| p),
            narration: payee_and_narration.map(|(_, n)| n),
        },
    ))
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
