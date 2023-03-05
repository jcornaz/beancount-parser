#![cfg(feature = "unstable")]

use crate::pest_parser::Pair;
use crate::{string, Date};

#[derive(Debug, Clone)]
pub struct Event<'a> {
    date: Date,
    name: &'a str,
    value: &'a str,
}

impl<'a> Event<'a> {
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

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
        let date = Date::from_pair(inner.next().expect("no date in event"));
        let name = string::from_pair(inner.next().expect("no name in event"));
        let value = string::from_pair(inner.next().expect("no value in event"));
        Self { date, name, value }
    }
}