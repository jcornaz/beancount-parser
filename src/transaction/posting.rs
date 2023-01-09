use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{char, space0, space1},
    combinator::{map, opt},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    account::{account, Account},
    amount::{amount, Amount},
    string::comment,
};

use super::{flag, Flag};

/// A posting
///
/// It is the association of an [`Account`] and an [`Amount`].
/// (though the amount is optional)
///
/// A posting may also have, price and cost defined after the amount.
///
/// # Examples of postings
///
/// * `Assets:A:B 10 CHF` (most common form)
/// * `! Assets:A:B 10 CHF` (with pending flag)
/// * `Assets:A:B 10 CHF @ 1 EUR` (with price)
/// * `Assets:A:B 10 CHF {2 USD}` (with cost)
/// * `Assets:A:B` (without amount)
#[derive(Debug, Clone, PartialEq)]
pub struct Posting<'a> {
    info: Info<'a>,
    amount: Option<Amount<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Info<'a> {
    pub(super) flag: Option<Flag>,
    pub(super) account: Account<'a>,
    pub(super) price: Option<(PriceType, Amount<'a>)>,
    pub(super) cost: Option<Amount<'a>>,
    pub(super) comment: Option<&'a str>,
}

impl<'a> Posting<'a> {
    /// Returns the flag on this posting (if present)
    #[must_use]
    pub fn flag(&self) -> Option<Flag> {
        self.info.flag
    }

    /// Returns the account referenced by this posting
    #[must_use]
    pub fn account(&self) -> &Account<'a> {
        &self.info.account
    }

    /// Returns the amount of the posting (if present)
    #[must_use]
    pub fn amount(&self) -> Option<&Amount<'a>> {
        self.amount.as_ref()
    }

    /// Returns a tuple of price-type and the price (if a price was defined)
    #[must_use]
    pub fn price(&self) -> Option<(PriceType, &Amount<'a>)> {
        self.info.price.as_ref().map(|(t, p)| (*t, p))
    }

    /// Returns the cost (if present)
    #[must_use]
    pub fn cost(&self) -> Option<&Amount<'a>> {
        self.info.cost.as_ref()
    }

    /// Returns the comment (if present)
    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        self.info.comment
    }
}

/// A price type
///
/// A price associated to an amount is either per-unit (`@`) or a total price (`@@`)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PriceType {
    /// Per-unit price
    Unit,
    /// Total price
    Total,
}

pub fn posting(input: &str) -> IResult<&str, Posting<'_>> {
    map(
        tuple((
            opt(terminated(flag, space1)),
            account,
            opt(preceded(space1, amount)),
            opt(preceded(
                space1,
                delimited(
                    tuple((char('{'), space0)),
                    amount,
                    tuple((
                        space0,
                        opt(tuple((char(','), take_till(|c| c == '}')))),
                        char('}'),
                    )),
                ),
            )),
            opt(preceded(space1, price)),
            opt(preceded(space0, comment)),
        )),
        |(flag, account, amount, cost, price, comment)| Posting {
            info: Info {
                flag,
                account,
                price,
                cost,
                comment,
            },
            amount,
        },
    )(input)
}

fn price(input: &str) -> IResult<&str, (PriceType, Amount<'_>)> {
    separated_pair(
        alt((
            map(tag("@@"), |_| PriceType::Total),
            map(char('@'), |_| PriceType::Unit),
        )),
        space1,
        amount,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Type as AccountType;

    #[test]
    fn simple_posting() {
        let input = "Assets:A:B 10 CHF";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(
            posting.account(),
            &Account::new(AccountType::Assets, ["A", "B"])
        );
        assert_eq!(posting.amount(), Some(&Amount::new(10, "CHF")));
        assert!(posting.price().is_none());
        assert!(posting.cost().is_none());
        assert!(posting.comment().is_none());
    }

    #[test]
    fn without_amount() {
        let input = "Assets:A:B";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert!(posting.amount().is_none());
    }

    #[test]
    fn with_price() {
        let input = "Assets:A:B 10 CHF @ 1 EUR";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(
            posting.price(),
            Some((PriceType::Unit, &Amount::new(1, "EUR")))
        );
    }

    #[test]
    fn with_total_price() {
        let input = "Assets:A:B 10 CHF @@ 9 EUR";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(
            posting.price(),
            Some((PriceType::Total, &Amount::new(9, "EUR")))
        );
    }

    #[rstest]
    fn with_cost(
        #[values("Assets:A:B 10 CHF {1 EUR}", "Assets:A:B 10 CHF { 1 EUR }")] input: &str,
    ) {
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(posting.cost(), Some(&Amount::new(1, "EUR")));
    }

    #[test]
    fn with_cost_and_date() {
        let input = "Assets:A:B 10 CHF {1 EUR, 2022-10-14}";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(posting.cost(), Some(&Amount::new(1, "EUR")));
    }

    #[test]
    fn with_cost_and_price() {
        let input = "Assets:A:B 10 CHF {2 USD} @ 1 EUR";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(posting.cost(), Some(&Amount::new(2, "USD")));
        assert_eq!(
            posting.price(),
            Some((PriceType::Unit, &Amount::new(1, "EUR")))
        );
    }

    #[test]
    fn with_flag() {
        let (_, posting) =
            posting("! Assets:A 1 EUR").expect("should successfully parse the posting");
        assert_eq!(posting.flag(), Some(Flag::Pending));
    }

    #[test]
    fn with_comment() {
        let input = "Assets:A:B 10 CHF ; Cool!";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(posting.comment(), Some("Cool!"));
    }

    #[rstest]
    fn invalid(#[values("")] input: &str) {
        assert!(posting(input).is_err());
    }
}
