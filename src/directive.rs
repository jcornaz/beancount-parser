use nom::{character::complete::space1, combinator::map, sequence::separated_pair, IResult};

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

pub(crate) fn directive(input: &str) -> IResult<&str, (Date, Directive<'_>)> {
    separated_pair(date, space1, map(transaction, Directive::Transaction))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction() {
        let input = r#"2022-09-10 txn "My transaction""#;
        let (_, (date, directive)) = directive(input).expect("should successfully parse directive");
        assert_eq!(date, Date::new(2022, 9, 10));
        match directive {
            Directive::Transaction(trx) => assert_eq!(trx.narration(), Some("My transaction")),
        }
    }
}
