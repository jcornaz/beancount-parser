#![allow(missing_docs)]

use nom::character::complete::space1;

use crate::{string, IResult, Span};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Event<S> {
    pub name: S,
    pub value: S,
}

pub(super) fn parse<'a, S: From<&'a str>>(input: Span<'a>) -> IResult<'a, Event<S>> {
    let (input, name) = string(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = string(input)?;
    Ok((input, Event { name, value }))
}
