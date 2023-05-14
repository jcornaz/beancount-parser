#![cfg(feature = "unstable")]

use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{char, space0};
use nom::sequence::delimited;

use crate::pest_parser::Pair;
use crate::{date::date, string, Date, IResult, Span};

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

    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let date = Date::from_pair(inner.next().expect("no date in event"));
        let name = string::from_pair(inner.next().expect("no name in event"));
        let value = string::from_pair(inner.next().expect("no value in event"));
        Self { date, name, value }
    }
}

pub(crate) fn event(input: Span<'_>) -> IResult<'_, Event<'_>> {
    let (input, date) = date(input)?;
    let (input, _) = delimited(space0, tag("event"), space0)(input)?;
    let (input, name) = delimited(char('"'), take_till(|c| c == '"'), char('"'))(input)?;
    let (input, _) = space0(input)?;
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
