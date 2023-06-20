use nom::{
    character::complete::{char, digit1},
    combinator::{map_res, verify},
    sequence::preceded,
};

use crate::{IResult, Span};

/// A date
///
/// The parser has some sanity check to make sure the date remotely makes sense
/// but it doesn't verify it is an actual real date valid date.
///
/// If that is important, you should use a date-time library to verify the validity.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Date {
    year: u16,
    month_of_year: u8,
    day_of_month: u8,
}

impl Date {
    /// Create a new date
    #[must_use]
    pub fn new(year: u16, month_of_year: u8, day_of_month: u8) -> Self {
        Self {
            year,
            month_of_year,
            day_of_month,
        }
    }

    /// Returns the year
    #[must_use]
    pub fn year(&self) -> u16 {
        self.year
    }

    /// Returns the number of the month in the year
    ///
    /// The result is between `1` (january) and `12` (december) inclusive.
    #[must_use]
    pub fn month_of_year(&self) -> u8 {
        self.month_of_year
    }

    /// Returns the number of the day in the month
    ///
    /// The result is between `1` and `31` inclusive
    #[must_use]
    pub fn day_of_month(&self) -> u8 {
        self.day_of_month
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.year
            .cmp(&other.year)
            .then_with(|| self.month_of_year.cmp(&other.month_of_year))
            .then_with(|| self.day_of_month.cmp(&other.day_of_month))
    }
}

pub(super) fn date(input: Span<'_>) -> IResult<'_, Date> {
    let (input, year) = year(input)?;
    let (input, month_of_year) = preceded(char('-'), month)(input)?;
    let (input, day_of_month) = preceded(char('-'), day)(input)?;
    Ok((
        input,
        Date {
            year,
            month_of_year,
            day_of_month,
        },
    ))
}

fn year(input: Span<'_>) -> IResult<'_, u16> {
    verify(map_res(digit1, |s: Span<'_>| s.fragment().parse()), |y| {
        *y > 0
    })(input)
}

fn month(input: Span<'_>) -> IResult<'_, u8> {
    verify(map_res(digit1, |s: Span<'_>| s.fragment().parse()), |m| {
        *m > 0 && *m <= 12
    })(input)
}

fn day(input: Span<'_>) -> IResult<'_, u8> {
    verify(map_res(digit1, |s: Span<'_>| s.fragment().parse()), |d| {
        *d > 0 && *d <= 31
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_date() {
        assert_eq!(
            date(Span::new("2022-08-15")).unwrap().1,
            Date {
                year: 2022,
                month_of_year: 8,
                day_of_month: 15
            }
        );
    }

    #[rstest]
    fn invalid_date(
        #[values(
            "hello",
            "0-1-1",
            "2000-00-12",
            "2000-13-12",
            "2000-11-00",
            "2000-11-32"
        )]
        input: &str,
    ) {
        assert!(date(Span::new(input)).is_err());
    }

    #[rstest]
    #[case(Date::new(2018, 11, 7), Date::new(2018, 11, 8))]
    #[case(Date::new(2018, 11, 8), Date::new(2018, 12, 7))]
    #[case(Date::new(2017, 11, 8), Date::new(2018, 11, 7))]
    fn date_comparison(#[case] before: Date, #[case] after: Date) {
        assert!(before < after);
        assert!(after > before);
    }
}
