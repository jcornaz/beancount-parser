use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, one_of, space1},
    combinator::{map, not, opt, value},
    sequence::{separated_pair, tuple},
    IResult,
};

use crate::{
    date::date,
    transaction::{transaction, Transaction},
    Date,
};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Directive<'a> {
    Transaction(Transaction<'a>),
}

pub(crate) fn directive(input: &str) -> IResult<&str, (Date, Option<Directive<'_>>)> {
    separated_pair(
        date,
        space1,
        alt((
            map(map(transaction, Directive::Transaction), Some),
            value(
                None,
                tuple((
                    not(tag("txn")),
                    not(one_of("*!")),
                    not_line_ending,
                    opt(line_ending),
                )),
            ),
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn transaction() {
        let input = r#"2022-09-10 txn "My transaction""#;
        let (_, (date, directive)) = directive(input).expect("should successfully parse directive");
        assert_eq!(date, Date::new(2022, 9, 10));
        match directive.expect("should recognize the directive") {
            Directive::Transaction(trx) => assert_eq!(trx.narration(), Some("My transaction")),
        }
    }

    #[rstest]
    #[case("2022-09-11 whatisthis \"hello\"", "")]
    #[case("2022-09-11 whatisthis \"hello\"\ntest", "test")]
    fn unkown_directive(#[case] input: &str, #[case] expected_rest: &str) {
        let (rest, _) = directive(input).expect("should successfully parse the directive");
        assert_eq!(rest, expected_rest);
    }

    #[rstest]
    fn invalid(
        #[values(
            "2022-09-11 txn that is incorrect",
            "2022-09-11 * that is incorrect",
            "2022-09-11 ! that is incorrect"
        )]
        input: &str,
    ) {
        assert!(directive(input).is_err());
    }
}
