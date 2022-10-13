use crate::amount::{amount, currency};
use crate::date::date;
use crate::{Amount, Date};
use nom::bytes::complete::tag;
use nom::character::streaming::space1;
use nom::combinator::map;
use nom::sequence::{delimited, tuple};
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
}

pub(crate) fn price(input: &str) -> IResult<&str, Price<'_>> {
    map(
        tuple((
            date,
            delimited(space1, tag("price"), space1),
            currency,
            space1,
            amount,
        )),
        |(date, _, commodity, _, price)| Price {
            date,
            commodity,
            price,
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
    }
}
