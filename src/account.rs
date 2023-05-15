use nom::{
    branch::alt,
    bytes::{complete::tag, complete::take_while},
    character::complete::{char, satisfy},
    combinator::{cut, recognize},
    multi::many1_count,
    sequence::preceded,
};

use super::{IResult, Span};

#[derive(Debug)]
pub struct Account<'a>(&'a str);

impl<'a> Account<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Account<'_>> {
    let (input, name) = recognize(preceded(
        alt((
            tag("Expenses"),
            tag("Assets"),
            tag("Liabilities"),
            tag("Income"),
            tag("Equity"),
        )),
        cut(many1_count(preceded(
            char(':'),
            preceded(
                satisfy(|c: char| c.is_uppercase()),
                take_while(|c: char| c.is_alphanumeric() || c == '-'),
            ),
        ))),
    ))(input)?;
    Ok((input, Account(name.fragment())))
}
