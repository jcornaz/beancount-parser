#![cfg(feature = "unstable")]

use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{char, space1};
use nom::sequence::delimited;

use crate::{date::date, Date, IResult, Span};

#[derive(Debug, Clone)]
pub struct Event<'a> {
    date: Date,
    name: &'a str,
    value: &'a str,
}

impl<'a> Event<'a> {
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    #[must_use]
    pub fn name(&self) -> &'a str {
        self.name
    }

    #[must_use]
    pub fn value(&self) -> &'a str {
        self.value
    }
}

pub(crate) fn event(input: Span<'_>) -> IResult<'_, Event<'_>> {
    let (input, date) = date(input)?;
    let (input, _) = delimited(space1, tag("event"), space1)(input)?;
    let (input, name) = delimited(char('"'), take_till(|c| c == '"'), char('"'))(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = delimited(char('"'), take_till(|c| c == '"'), char('"'))(input)?;
    Ok((
        input,
        Event {
            date,
            name: name.fragment(),
            value: value.fragment(),
        },
    ))
}
