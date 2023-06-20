#![cfg(feature = "unstable")]

use nom::{bytes::complete::tag, character::complete::space1, sequence::delimited};

use crate::{amount::currency, date::date, IResult, Span};

/// The commodity declaration directive
///
/// See: <https://beancount.github.io/docs/beancount_language_syntax.html#commodity>
#[derive(Debug, Clone)]
pub struct Commodity<'a> {
    currency: &'a str,
}

impl<'a> Commodity<'a> {
    /// Currency being declared
    #[must_use]
    pub fn currency(&self) -> &'a str {
        self.currency
    }
}

pub(crate) fn commodity(input: Span<'_>) -> IResult<'_, Commodity<'_>> {
    let (input, _) = date(input)?;
    let (input, _) = delimited(space1, tag("commodity"), space1)(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Commodity { currency }))
}
