use std::{cmp::Ordering, str::FromStr};

use crate::v3::error::ParseError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    #[must_use]
    pub fn year(self) -> u16 {
        self.year
    }

    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }

    #[must_use]
    pub fn day(self) -> u8 {
        self.day
    }

    #[must_use]
    #[allow(clippy::manual_range_contains)]
    pub fn from_ymd(year: u16, month: u8, day: u8) -> Option<Self> {
        if year < 10_000
            && month >= 1
            && month <= 12
            && day >= 1
            && day <= month_length(year, month)
        {
            Some(Self { year, month, day })
        } else {
            None
        }
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.year.cmp(&other.year) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.month.cmp(&other.month) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.day.cmp(&other.day)
    }
}

impl FromStr for Date {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s).ok_or(ParseError)
    }
}

#[must_use]
pub fn parse(input: &str) -> Option<Date> {
    if input.len() != 10
        || input[4..5] != input[7..8]
        || (&input[4..5] != "-" && &input[4..5] != "/")
    {
        return None;
    }
    let year: u16 = input[..4].parse().ok()?;
    let month: u8 = input[5..7].parse().ok()?;
    let day: u8 = input[8..].parse().ok()?;
    Date::from_ymd(year, month, day)
}

fn month_length(year: u16, month: u8) -> u8 {
    match month {
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}

fn is_leap_year(year: u16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("2024-02-29", Date::from_ymd(2024, 2, 29).unwrap())]
    #[case("2026-06-28", Date::from_ymd(2026, 6, 28).unwrap())]
    #[case("2026/06/28", Date::from_ymd(2026, 6, 28).unwrap())]
    fn should_parse_valid_date(#[case] input: &str, #[case] expected: Date) {
        let date: Date = input
            .parse()
            .unwrap_or_else(|_| panic!("'{input}' should be a valid date"));
        assert_eq!(date, expected);
    }

    #[rstest]
    #[case::empty("")]
    #[case::wtf("oops")]
    #[case::month_does_not_exist("2026-00-01")]
    #[case::month_does_not_exist("2026-13-01")]
    #[case::day_does_not_exist("2026-01-00")]
    #[case::day_does_not_exist("2026-02-29")]
    #[case::day_does_not_exist("2026-03-32")]
    #[case::day_does_not_exist("2026-04-31")]
    #[case::day_does_not_exist("2026-05-32")]
    #[case::day_does_not_exist("2026-06-31")]
    #[case::day_does_not_exist("2026-07-32")]
    #[case::day_does_not_exist("2026-08-32")]
    #[case::day_does_not_exist("2026-09-31")]
    #[case::day_does_not_exist("2026-10-32")]
    #[case::day_does_not_exist("2026-11-31")]
    #[case::day_does_not_exist("2026-12-32")]
    #[case::separator_mix("2026-01/01")]
    #[case::one_digit_month("2026-6-32")]
    #[case::one_digit_day("2026-06-1")]
    #[case::suffix("2026-01-01-01")]
    fn should_fail_to_parse_invalid_date(#[case] input: &str) {
        Date::from_str(input).expect_err(&format!("'{input}' should not be a valid date"));
    }

    #[rstest]
    #[case(2000)]
    #[case(2024)]
    #[case(1600)]
    fn test_leap_year(#[case] year: u16) {
        assert!(is_leap_year(year), "{year} should be a leap year");
    }

    #[rstest]
    #[case(2026)]
    #[case(1900)]
    fn test_not_leap_year(#[case] year: u16) {
        assert!(!is_leap_year(year), "{year} should not be a leap year");
    }
}
