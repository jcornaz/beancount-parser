use super::{
    account::{self, Account},
    IResult, Span,
};

#[derive(Debug)]
pub struct Open<'a> {
    pub account: Account<'a>,
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Open<'_>> {
    let (input, account) = account::parse(input)?;
    Ok((input, Open { account }))
}
