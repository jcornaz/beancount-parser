use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

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
