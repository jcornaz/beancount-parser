use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space1},
    combinator::{map, opt},
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    account::{account, Account},
    amount::{amount, Amount},
};

use super::{flag, Flag};

#[derive(Debug, Clone, PartialEq)]
pub struct Posting<'a> {
    flag: Option<Flag>,
    account: Account<'a>,
    amount: Option<Amount<'a>>,
    price: Option<(PriceType, Amount<'a>)>,
}

impl<'a> Posting<'a> {
    pub fn flag(&self) -> Option<Flag> {
        self.flag
    }

    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    pub fn amount(&self) -> Option<&Amount<'a>> {
        self.amount.as_ref()
    }

    pub fn price(&self) -> Option<(PriceType, &Amount<'a>)> {
        self.price.as_ref().map(|(t, p)| (*t, p))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PriceType {
    UnitPrice,
    TotalCost,
}

pub fn posting(input: &str) -> IResult<&str, Posting<'_>> {
    map(
        tuple((
            opt(terminated(flag, space1)),
            account,
            opt(preceded(space1, amount)),
            opt(preceded(space1, price)),
        )),
        |(flag, account, amount, price)| Posting {
            flag,
            account,
            amount,
            price,
        },
    )(input)
}

fn price(input: &str) -> IResult<&str, (PriceType, Amount<'_>)> {
    separated_pair(
        alt((
            map(tag("@@"), |_| PriceType::TotalCost),
            map(char('@'), |_| PriceType::UnitPrice),
        )),
        space1,
        amount,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Type as AccountType;
    use rstest::rstest;

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
            Some((PriceType::UnitPrice, &Amount::new(1, "EUR")))
        )
    }

    #[test]
    fn with_total_price() {
        let input = "Assets:A:B 10 CHF @@ 9 EUR";
        let (_, posting) = posting(input).expect("should successfully parse the posting");
        assert_eq!(
            posting.price(),
            Some((PriceType::TotalCost, &Amount::new(9, "EUR")))
        )
    }

    #[test]
    fn with_flag() {
        let (_, posting) =
            posting("! Assets:A 1 EUR").expect("should succesfully parse the posting");
        assert_eq!(posting.flag(), Some(Flag::Pending));
    }

    #[rstest]
    fn invalid(#[values("")] input: &str) {
        assert!(posting(input).is_err());
    }
}
