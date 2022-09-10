use nom::{
    character::complete::{char, digit1},
    combinator::{map_res, verify},
    sequence::preceded,
    IResult,
};

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
