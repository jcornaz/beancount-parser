use crate::amount::{amount, currency};
use crate::date::date;
use crate::string::comment;
use crate::{Amount, Date};
use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::character::streaming::space1;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

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
    pub fn date(&self) -> Date {
        self.date
    }

    /// The commodity for which thi price applies
    pub fn commodity(&self) -> &'a str {
        self.commodity
    }

    /// The price of the commodity
    pub fn price(&self) -> &Amount<'a> {
        &self.price
    }

    pub fn comment(&self) -> Option<&'a str> {
        self.comment
    }
}

pub(crate) fn price(input: &str) -> IResult<&str, Price<'_>> {
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
        let (_, price) = price(input).expect("should successfully parse the input");
        assert_eq!(price.date(), Date::new(2014, 7, 9));
        assert_eq!(price.commodity(), "HOOL");
        assert_eq!(price.price(), &Amount::new(600, "USD"));
        assert_eq!(price.comment(), None);
    }

    #[test]
    fn comment() {
        let input = "2014-07-09 price HOOL  600 USD ; with comment";
        let (_, price) = price(input).expect("should successfully parse the input");
        assert_eq!(price.comment(), Some("with comment"));
    }
}
