use nom::character::complete::space1;

use crate::{string, IResult, Span};

#[derive(Debug)]
#[non_exhaustive]
pub struct Event<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Event<'_>> {
    let (input, name) = string(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = string(input)?;
    Ok((input, Event { name, value }))
}
