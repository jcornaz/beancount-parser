use nom::{
    character::complete::{char, digit1},
    combinator::{map_res, verify},
    sequence::preceded,
    IResult,
};

use crate::Date;

pub(super) fn date(input: &str) -> IResult<&str, Date> {
    let (input, year) = year(input)?;
    let (input, month_of_year) = preceded(char('-'), month)(input)?;
    let (input, day_of_month) = preceded(char('-'), day)(input)?;
    Ok((
        input,
        Date {
            year,
            day_of_month,
            month_of_year,
        },
    ))
}

fn year(input: &str) -> IResult<&str, u16> {
    verify(map_res(digit1, |s: &str| s.parse()), |y| *y > 0)(input)
}

fn month(input: &str) -> IResult<&str, u8> {
    verify(map_res(digit1, |s: &str| s.parse()), |m| *m > 0 && *m <= 12)(input)
}

fn day(input: &str) -> IResult<&str, u8> {
    verify(map_res(digit1, |s: &str| s.parse()), |d| *d > 0 && *d <= 31)(input)
}

struct InvalidDate {}

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
