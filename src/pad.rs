#![cfg(test)]

use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::map,
    sequence::{terminated, tuple},
    IResult,
};

use crate::{account::account, date::date, Account, Date};

pub struct Pad<'a> {
    date: Date,
    target_account: Account<'a>,
    source_account: Account<'a>,
}

impl<'a> Pad<'a> {
    pub fn date(&self) -> Date {
        self.date
    }

    pub fn target_account(&self) -> &Account<'a> {
        &self.target_account
    }

    pub fn source_account(&self) -> &Account<'a> {
        &self.source_account
    }
}

pub(crate) fn pad(input: &str) -> IResult<&str, Pad<'_>> {
    map(
        tuple((
            terminated(date, tuple((space1, tag("pad"), space1))),
            terminated(account, space1),
            account,
        )),
        |(date, target_account, source_account)| Pad {
            date,
            target_account,
            source_account,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::account;

    use super::*;

    use nom::combinator::all_consuming;

    #[test]
    fn valid_pad() {
        let input = "2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances";
        let (_, pad) = all_consuming(pad)(input).unwrap();
        assert_eq!(pad.date(), Date::new(2014, 6, 1));
        assert_eq!(
            pad.target_account(),
            &Account::new(account::Type::Assets, ["BofA", "Checking"])
        );
        assert_eq!(
            pad.source_account(),
            &Account::new(account::Type::Equity, ["Opening-Balances"])
        );
    }
}
