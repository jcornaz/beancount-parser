use nom::{
    character::complete::{alphanumeric1, char},
    combinator::recognize,
    multi::separated_list1,
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
    let (input, name) = recognize(separated_list1(char(':'), alphanumeric1))(input)?;
    Ok((input, Account(name.fragment())))
}
