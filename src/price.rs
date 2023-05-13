use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::character::streaming::space1;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, terminated, tuple};

use crate::amount::{amount, currency};
use crate::date::date;
#[cfg(feature = "unstable")]
use crate::pest_parser::Pair;
use crate::string::comment;
use crate::{Amount, Date, IResult, Span};

/// A price directive
///
/// # Example
/// ```beancount
/// 2014-07-09 price HOOL  579.18 USD
/// ```
#[derive(Debug, Clone)]
pub struct Price<'a> {
    date: Date,
    commodity: &'a str,
    price: Amount<'a>,
    comment: Option<&'a str>,
}

impl<'a> Price<'a> {
    /// The date
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// The commodity for which thi price applies
    #[must_use]
    pub fn commodity(&self) -> &'a str {
        self.commodity
    }

    /// The price of the commodity
    #[must_use]
    pub fn price(&self) -> &Amount<'a> {
        &self.price
    }

    /// The comment, if any
    #[must_use]
    pub fn comment(&self) -> Option<&'a str> {
        self.comment
    }

    #[cfg(feature = "unstable")]
    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let date = Date::from_pair(inner.next().expect("no date in price directive"));
        let commodity = inner
            .next()
            .expect("no commodity in price directive")
            .as_str();
        let price = Amount::from_pair(inner.next().expect("no amount in price directive"));
        Self {
            commodity,
            price,
            date,
            comment: None,
        }
    }
}

pub(crate) fn price(input: Span<'_>) -> IResult<'_, Price<'_>> {
    map(
        tuple((
            terminated(date, tuple((space1, tag("price"), space1))),
            terminated(currency, space1),
            amount,
            opt(preceded(space0, comment)),
        )),
        |(date, commodity, price, comment)| Price {
            date,
            commodity,
            price,
            comment,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let input = "2014-07-09 price HOOL  600 USD";
        let (_, price) = price(Span::new(input)).expect("should successfully parse the input");
        assert_eq!(price.date(), Date::new(2014, 7, 9));
        assert_eq!(price.commodity(), "HOOL");
        assert_eq!(price.price(), &Amount::new(600, "USD"));
        assert_eq!(price.comment(), None);
    }

    #[test]
    fn comment() {
        let input = "2014-07-09 price HOOL  600 USD ; with comment";
        let (_, price) = price(Span::new(input)).expect("should successfully parse the input");
        assert_eq!(price.comment(), Some("with comment"));
    }
}
