use std::{cmp::Ordering, str::FromStr};

use nom::{
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::{all_consuming, cut, map_res, peek, verify},
    sequence::tuple,
    Finish,
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

impl Date {
    /// Create a new date from year, month and day
    #[must_use]
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
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

impl FromStr for Date {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let span = Span::new(s);
        match all_consuming(parse)(span).finish() {
            Ok((_, date)) => Ok(date),
            Err(_) => Err(crate::Error::new(s, span)),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_from_str_should_parse_valid_date() {
        let date: Date = "2023-03-12".parse().unwrap();
        assert_eq!(date.year, 2023);
        assert_eq!(date.month, 3);
        assert_eq!(date.day, 12);
    }

    #[test]
    fn date_from_str_should_not_parse_invalid_date() {
        let result: Result<Date, _> = "2023-03-12oops".parse();
        assert!(result.is_err(), "{result:?}");
    }
}

#[cfg(test)]
pub(crate) mod chumsky {
    use crate::{ChumskyError, ChumskyParser, Date};

    use chumsky::prelude::*;

    pub(crate) fn date() -> impl ChumskyParser<Date> {
        year()
            .then_ignore(just('-'))
            .then(month())
            .then_ignore(just('-'))
            .then(day())
            .map(|((year, month), day)| Date::new(year, month, day))
            .labelled("date")
    }

    fn year() -> impl ChumskyParser<u16> {
        filter(|c: &char| c.is_ascii_digit())
            .repeated()
            .exactly(4)
            .collect::<String>()
            .from_str()
            .unwrapped()
            .labelled("year")
    }

    fn month() -> impl ChumskyParser<u8> {
        filter(|c: &char| c.is_ascii_digit())
            .repeated()
            .exactly(2)
            .collect::<String>()
            .from_str()
            .unwrapped()
            .validate(|m: u8, span, emit| {
                if m == 0 || m > 12 {
                    emit(ChumskyError::custom(span, "must be between 1 and 12"));
                }
                m
            })
            .labelled("month")
    }

    fn day() -> impl ChumskyParser<u8> {
        filter(|c: &char| c.is_ascii_digit())
            .repeated()
            .exactly(2)
            .collect::<String>()
            .from_str()
            .unwrapped()
            .validate(|d: u8, span, emit| {
                if d == 0 || d > 31 {
                    emit(ChumskyError::custom(span, "must be between 1 and 31"));
                }
                d
            })
            .labelled("day")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rstest::rstest;

        #[rstest]
        #[case::first_day_of_year("2023-01-01", Date::new(2023, 1, 1))]
        #[case::last_day_of_year("2023-12-31", Date::new(2023, 12, 31))]
        fn should_parse_valid_date(#[case] input: &str, #[case] expected: Date) {
            let date: Date = date().then_ignore(end()).parse(input).unwrap();
            assert_eq!(date, expected);
        }

        #[rstest]
        #[case("2023-13-01")]
        #[case("2023-10-32")]
        #[case("2023-01-00")]
        #[case("2023-00-01")]
        #[case("2023-1-2")]
        #[case("2023-01-2")]
        #[case("2023-1-02")]
        #[case("23-01-02")]
        fn should_not_parse_invalid_date(#[case] input: &str) {
            let result: Result<Date, _> = date().then_ignore(end()).parse(input);
            assert!(result.is_err(), "{result:?}");
        }
    }
}
