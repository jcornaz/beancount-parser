use nom::character::complete::not_line_ending;

use super::{IResult, Span};

#[derive(Debug)]
pub struct Account<'a>(&'a str);

impl<'a> Account<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Account<'_>> {
    let (input, name) = not_line_ending(input)?;
    Ok((input, Account(name.fragment())))
}
