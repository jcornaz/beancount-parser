use nom::{
    character::complete::{char, space0},
    multi::separated_list0,
    sequence::delimited,
};

use crate::{
    account::{self, Account},
    currency::{self, Currency},
    IResult, Span,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct Open<'a> {
    pub account: Account<'a>,
    pub currencies: Vec<Currency<'a>>,
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Open<'_>> {
    let (input, account) = account::parse(input)?;
    let (input, _) = space0(input)?;
    let sep = delimited(space0, char(','), space0);
    let (input, currencies) = separated_list0(sep, currency::parse)(input)?;
    Ok((
        input,
        Open {
            account,
            currencies,
        },
    ))
}
