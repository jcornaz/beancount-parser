use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::{complete::tag, complete::take_while},
    character::complete::{char, satisfy, space0, space1},
    combinator::{cut, recognize},
    multi::{many1_count, separated_list0},
    sequence::{delimited, preceded},
};

use crate::amount::{self, Amount, Currency};

use super::{IResult, Span};

#[derive(Debug)]
pub struct Account<'a>(&'a str);

impl<'a> Account<'a> {
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Open<'a> {
    pub account: Account<'a>,
    pub currencies: Vec<Currency<'a>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Close<'a> {
    pub account: Account<'a>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Balance<'a, D> {
    pub account: Account<'a>,
    pub amount: Amount<'a, D>,
}

/// [pad] directive
///
/// [pad]: https://beancount.github.io/docs/beancount_language_syntax.html#pad
///
/// # Example
/// ```
/// use beancount_parser_2::DirectiveContent;
/// let raw = "2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances";
/// let file = beancount_parser_2::parse::<f64>(raw).unwrap();
/// let DirectiveContent::Pad(pad) = &file.directives[0].content else { unreachable!() };
/// assert_eq!(pad.account.as_str(), "Assets:BofA:Checking");
/// assert_eq!(pad.source_account.as_str(), "Equity:Opening-Balances");
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub struct Pad<'a> {
    pub account: Account<'a>,
    pub source_account: Account<'a>,
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
                satisfy(char::is_uppercase),
                take_while(|c: char| c.is_alphanumeric() || c == '-'),
            ),
        ))),
    ))(input)?;
    Ok((input, Account(name.fragment())))
}

pub(super) fn open(input: Span<'_>) -> IResult<'_, Open<'_>> {
    let (input, account) = parse(input)?;
    let (input, _) = space0(input)?;
    let sep = delimited(space0, char(','), space0);
    let (input, currencies) = separated_list0(sep, amount::currency)(input)?;
    Ok((
        input,
        Open {
            account,
            currencies,
        },
    ))
}

pub(super) fn close(input: Span<'_>) -> IResult<'_, Close<'_>> {
    let (input, account) = parse(input)?;
    Ok((input, Close { account }))
}

pub(super) fn balance<D: FromStr>(input: Span<'_>) -> IResult<'_, Balance<'_, D>> {
    let (input, account) = parse(input)?;
    let (input, _) = space1(input)?;
    let (input, amount) = amount::parse(input)?;
    Ok((input, Balance { account, amount }))
}

pub(super) fn pad(input: Span<'_>) -> IResult<'_, Pad<'_>> {
    let (input, account) = parse(input)?;
    let (input, _) = space1(input)?;
    let (input, source_account) = parse(input)?;
    Ok((
        input,
        Pad {
            account,
            source_account,
        },
    ))
}
