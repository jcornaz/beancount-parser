#![cfg(feature = "unstable")]

use crate::pest_parser::Pair;
use crate::string;

/// beancount option
///
/// see: <https://beancount.github.io/docs/beancount_language_syntax.html#options>
#[derive(Debug, Clone)]
pub struct Option<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> Option<'a> {
    #[must_use]
    pub fn name(&self) -> &'a str {
        self.name
    }

    #[must_use]
    pub fn value(&self) -> &'a str {
        self.value
    }

    pub(crate) fn from_pair(pair: Pair<'a>) -> Self {
        let mut inner = pair.into_inner();
        let name = string::from_pair(inner.next().expect("no name in option"));
        let value = string::from_pair(inner.next().expect("no name in option"));
        Self { name, value }
    }
}
