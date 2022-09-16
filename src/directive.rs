use nom::{
    branch::alt,
    character::complete::{line_ending, not_line_ending, space1},
    combinator::{map, opt, value},
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

impl<'a> Directive<'a> {
    #[must_use]
    pub fn as_transaction(&self) -> Option<&Transaction<'a>> {
        match self {
            Directive::Transaction(trx) => Some(trx),
        }
    }
}

pub(crate) fn directive(input: &str) -> IResult<&str, (Date, Option<Directive<'_>>)> {
    alt((
        map(transaction, |trx| {
            (trx.date(), Some(Directive::Transaction(trx)))
        }),
        separated_pair(
            date,
            space1,
            value(None, tuple((not_line_ending, opt(line_ending)))),
        ),
    ))(input)
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
        let transaction = directive
            .as_ref()
            .expect("should recognize the directive")
            .as_transaction()
            .expect("the directive should be a transaction");
        assert_eq!(transaction.narration(), Some("My transaction"));
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
