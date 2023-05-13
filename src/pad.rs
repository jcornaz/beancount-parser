use nom::{
    bytes::complete::tag,
    character::complete::space1,
    combinator::cut,
    sequence::{preceded, terminated, tuple},
};

use crate::{account::account, date::date, Account, Date, IResult, Span};

/// Padding directive
///
/// The padding directive is an instruction to automatically
/// insert a transaction that will make the next balance assertion to succeed.
///
/// See: <https://beancount.github.io/docs/beancount_language_syntax.html#pad>
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pad<'a> {
    date: Date,
    target_account: Account<'a>,
    source_account: Account<'a>,
}

impl<'a> Pad<'a> {
    /// Date of the pad
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// Account to credit on the next balance assertion
    ///
    /// It is the account that must have a balance assertion for the pad to be effective
    ///
    /// It is the first account mentionned in the directive
    #[must_use]
    pub fn target_account(&self) -> &Account<'a> {
        &self.target_account
    }

    /// Source of the founds when the [`Self::target_account`] is padded
    ///
    /// It is the second account mentionned in the directive
    #[must_use]
    pub fn source_account(&self) -> &Account<'a> {
        &self.source_account
    }
}

pub(crate) fn pad(input: Span<'_>) -> IResult<'_, Pad<'_>> {
    let (input, date) = terminated(date, tuple((space1, tag("pad"))))(input)?;
    let (input, target_account) = cut(preceded(space1, account))(input)?;
    let (input, source_account) = cut(preceded(space1, account))(input)?;
    Ok((
        input,
        Pad {
            date,
            target_account,
            source_account,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::account;

    use super::*;

    use nom::combinator::all_consuming;

    #[test]
    fn valid_pad() {
        let input = "2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances";
        let (_, pad) = all_consuming(pad)(Span::new(input)).unwrap();
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

    #[rstest]
    fn invalid_pad(
        #[values(
            "2014-06-01 pad",
            "2014-06-01 pad oops",
            "2014-06-01 pad Assets:Test",
            "2014-06-01 pad Assets:Test oops"
        )]
        input: &str,
    ) {
        let res = pad(Span::new(input));
        assert!(matches!(res, Err(nom::Err::Failure(_))), "{res:?}");
    }

    #[rstest]
    fn not_a_pad(#[values("", "2014-06-01 txn", "hello", "; hello")] input: &str) {
        let res = pad(Span::new(input));
        assert!(matches!(res, Err(nom::Err::Error(_))), "{res:?}");
    }
}
