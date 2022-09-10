use nom::{
    branch::alt,
    bytes::complete::tag,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Posting<'a> {
    flag: Option<Flag>,
    account: Account<'a>,
    amount: Option<Amount<'a>>,
    price: Option<(PriceType, Amount<'a>)>,
    cost: Option<Amount<'a>>,
    comment: Option<&'a str>,
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

    pub fn cost(&self) -> Option<&Amount<'a>> {
        self.cost.as_ref()
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PriceType {
    Unit,
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
                    tuple((space0, char('}'))),
                ),
            )),
            opt(preceded(space1, price)),
            opt(preceded(space0, comment)),
        )),
        |(flag, account, amount, cost, price, comment)| Posting {
            flag,
            account,
            amount,
            price,
            cost,
            comment,
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
            posting("! Assets:A 1 EUR").expect("should succesfully parse the posting");
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
