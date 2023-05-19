use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

#[cfg(feature = "unstable")]
use crate::pest_parser::{Pair, Rule};
use crate::{
    account::{account, Account},
    amount::{amount, Amount},
    string::{comment, string},
    IResult, Span,
};

use super::{date, flag, Date, Flag};

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
    flag: Option<Flag>,
    account: Account<'a>,
    amount: Option<Amount<'a>>,
    price: Option<(PriceType, Amount<'a>)>,
    lot: Option<LotAttributes<'a>>,
    comment: Option<&'a str>,
}

impl<'a> Posting<'a> {
    /// Returns the flag on this posting (if present)
    #[must_use]
    pub fn flag(&self) -> Option<Flag> {
        self.flag
    }

    /// Returns the account referenced by this posting
    #[must_use]
    pub fn account(&self) -> &Account<'a> {
        &self.account
    }

    /// Returns the amount of the posting (if present)
    #[must_use]
    pub fn amount(&self) -> Option<&Amount<'a>> {
        self.amount.as_ref()
    }

    /// Returns the lot identifier (declared within '{' and '}')
    #[must_use]
    #[cfg(all(test, feature = "unstable"))]
    pub(crate) fn lot(&self) -> Option<&LotAttributes<'a>> {
        self.lot.as_ref()
    }

    /// Returns a tuple of price-type and the price (if a price was defined)
    #[must_use]
    pub fn price(&self) -> Option<(PriceType, &Amount<'a>)> {
        self.price.as_ref().map(|(t, p)| (*t, p))
    }

    /// Returns the cost (if present)
    #[must_use]
    pub fn cost(&self) -> Option<&Amount<'a>> {
        self.lot.as_ref().and_then(|la| la.cost.as_ref())
    }

    /// Returns the comment (if present)
    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        self.comment
    }

    #[cfg(feature = "unstable")]
    pub(super) fn from_pair(pair: Pair<'a>) -> Self {
        let mut flag = None;
        let mut account = None;
        let mut amount = None;
        let mut comment = None;
        let mut lot = None;
        let mut price = None;
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::account => account = Some(Account::from_pair(pair)),
                Rule::amount => amount = Some(Amount::from_pair(pair)),
                Rule::transaction_flag => flag = Some(Flag::from_pair(pair)),
                Rule::comment => comment = Some(pair.as_str()),
                Rule::lot => lot = Some(LotAttributes::from_pair(pair)),
                Rule::price => price = Some(price_from_pair(pair)),
                _ => (),
            }
        }
        Posting {
            flag,
            account: account.expect("no account in posting"),
            price,
            lot,
            comment,
            amount,
        }
    }
}

#[cfg(feature = "unstable")]
fn price_from_pair(pair: Pair<'_>) -> (PriceType, Amount<'_>) {
    let mut inner = pair.into_inner();
    let type_ = match inner.next().expect("not price type").as_str() {
        "@" => PriceType::Unit,
        "@@" => PriceType::Total,
        _ => unreachable!("invalid price type"),
    };
    let amount = Amount::from_pair(inner.next().expect("no amount in price"));
    (type_, amount)
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LotAttributes<'a> {
    cost: Option<Amount<'a>>,
    date: Option<Date>,
    label: Option<String>,
}

enum LotAttribute<'a> {
    Cost(Amount<'a>),
    Date(Date),
    Label(String),
}

impl<'a> LotAttributes<'a> {
    #[cfg(all(test, feature = "unstable"))]
    pub(crate) fn cost(&self) -> Option<&Amount<'a>> {
        self.cost.as_ref()
    }

    #[cfg(all(test, feature = "unstable"))]
    pub(crate) fn date(&self) -> Option<Date> {
        self.date
    }

    #[cfg(feature = "unstable")]
    fn from_pair(pair: Pair<'a>) -> Self {
        let mut cost = None;
        let mut date = None;
        let label = None;
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::amount => cost = Some(Amount::from_pair(pair)),
                Rule::date => date = Some(Date::from_pair(pair)),
                _ => unreachable!("unexpected token in lot attributes"),
            }
        }
        Self { cost, date, label }
    }
}

fn lot_attributes(input: Span<'_>) -> IResult<'_, LotAttributes<'_>> {
    let (input, attrs) = separated_list0(
        tuple((space0, char(','), space0)),
        alt((
            map(amount, LotAttribute::Cost),
            map(date, LotAttribute::Date),
            map(string, LotAttribute::Label),
        )),
    )(input)?;

    Ok((
        input,
        attrs.iter().fold(
            LotAttributes {
                cost: None,
                date: None,
                label: None,
            },
            |acc, attr| match attr {
                LotAttribute::Cost(c) => LotAttributes {
                    cost: Some(c.clone()),
                    ..acc
                },
                LotAttribute::Date(d) => LotAttributes {
                    date: Some(*d),
                    ..acc
                },
                LotAttribute::Label(s) => LotAttributes {
                    label: Some(s.to_string()),
                    ..acc
                },
            },
        ),
    ))
}

pub fn posting(input: Span<'_>) -> IResult<'_, Posting<'_>> {
    let (input, flag) = opt(terminated(flag, space1))(input)?;
    let (input, account) = account(input)?;
    let (input, amount) = opt(preceded(space1, amount))(input)?;
    let (input, lot) = opt(preceded(
        space1,
        delimited(
            tuple((char('{'), space0)),
            lot_attributes,
            preceded(space0, char('}')),
        ),
    ))(input)?;
    let (input, price) = opt(preceded(space1, price))(input)?;
    let (input, _) = space0(input)?;
    let (input, comment) = opt(comment)(input)?;
    Ok((
        input,
        Posting {
            flag,
            account,
            amount,
            price,
            lot,
            comment,
        },
    ))
}

fn price(input: Span<'_>) -> IResult<'_, (PriceType, Amount<'_>)> {
    separated_pair(
        alt((
            map(tag("@@"), |_| PriceType::Total),
            map(char('@'), |_| PriceType::Unit),
        )),
        space0,
        amount,
    )(input)
}
