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
    pub narration: Option<&'a str>,
}

pub(crate) fn parse(input: Span<'_>) -> IResult<'_, Transaction<'_>> {
    let (input, _) = tag("txn")(input)?;
    cut(do_parse)(input)
}

fn do_parse(input: Span<'_>) -> IResult<'_, Transaction<'_>> {
    let (input, s1) = opt(preceded(space1, string))(input)?;
    let (input, s2) = opt(preceded(space1, string))(input)?;
    Ok((
        input,
        Transaction {
            narration: s2.or(s1),
        },
    ))
}
