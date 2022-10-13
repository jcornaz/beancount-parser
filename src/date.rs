use nom::{
    character::complete::{char, digit1},
    combinator::{map_res, verify},
    sequence::preceded,
    IResult,
};

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
    #[cfg(test)]
    pub(crate) fn new(year: u16, month_of_year: u8, day_of_month: u8) -> Self {
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

pub(super) fn date(input: &str) -> IResult<&str, Date> {
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

fn year(input: &str) -> IResult<&str, u16> {
    verify(map_res(digit1, str::parse), |y| *y > 0)(input)
}

fn month(input: &str) -> IResult<&str, u8> {
    verify(map_res(digit1, str::parse), |m| *m > 0 && *m <= 12)(input)
}

fn day(input: &str) -> IResult<&str, u8> {
    verify(map_res(digit1, str::parse), |d| *d > 0 && *d <= 31)(input)
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::rstest;

    #[test]
    fn valid_date() {
        assert_eq!(
            date("2022-08-15"),
            Ok((
                "",
                Date {
                    year: 2022,
                    month_of_year: 8,
                    day_of_month: 15
                }
            )),
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
        assert!(date(input).is_err());
    }
}
