use std::cmp::Ordering;

use nom::{
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::{cut, map_res, peek, verify},
    sequence::tuple,
};

use super::{IResult, Span};

/// A date
///
/// The parser has some sanity checks to make sure the date remotely makes sense
/// but it doesn't verify if it is an actual real valid date.
///
/// If that is important to you, you should use a date-time library to verify the validity.
///
/// # Example
///
/// ```
/// # use beancount_parser::BeancountFile;
/// let input = "2022-05-21 event \"location\" \"Middle earth\"";
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let date = beancount.directives[0].date;
/// assert_eq!(date.year, 2022);
/// assert_eq!(date.month, 5);
/// assert_eq!(date.day, 21);
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Date {
    /// Year
    pub year: u16,
    /// Month (of year)
    pub month: u8,
    /// Day (of month)
    pub day: u8,
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Date> {
    let (input, _) = peek(tuple((digit1, char('-'), digit1, char('-'), digit1)))(input)?;
    cut(do_parse)(input)
}

fn do_parse(input: Span<'_>) -> IResult<'_, Date> {
    let (input, year) = year(input)?;
    let (input, _) = char('-')(input)?;
    let (input, month) = month(input)?;
    let (input, _) = char('-')(input)?;
    let (input, day) = day(input)?;
    Ok((input, Date { year, month, day }))
}

fn year(input: Span<'_>) -> IResult<'_, u16> {
    map_res(take(4usize), |s: Span<'_>| s.fragment().parse())(input)
}

fn month(input: Span<'_>) -> IResult<'_, u8> {
    verify(
        map_res(take(2usize), |s: Span<'_>| s.fragment().parse()),
        |&n| n > 0 && n < 13,
    )(input)
}

fn day(input: Span<'_>) -> IResult<'_, u8> {
    verify(
        map_res(take(2usize), |s: Span<'_>| s.fragment().parse()),
        |&n| n > 0 && n < 32,
    )(input)
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        self.year
            .cmp(&other.year)
            .then_with(|| self.month.cmp(&other.month))
            .then_with(|| self.day.cmp(&other.day))
    }
}
