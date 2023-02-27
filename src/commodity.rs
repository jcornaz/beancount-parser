#![cfg(feature = "unstable")]

use crate::pest_parser::Pair;

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

    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let _date = inner.next().expect("no date in commodity declaration");
        let currency = inner
            .next()
            .expect("no currency in commodity declaration")
            .as_str();
        Self { currency }
    }
}
