#![cfg(feature = "unstable")]

use std::{
    collections::HashMap,
    iter::Sum,
    ops::{Add, AddAssign},
};

use rust_decimal::Decimal;

use super::Amount;

/// An aggregagion of [`Amount`]
#[derive(Debug, Default, Clone)]
pub struct Amounts<'a>(HashMap<&'a str, Decimal>);

impl<'a> Amounts<'a> {
    pub fn iter(&self) -> impl Iterator<Item = Amount<'a>> + '_ {
        self.0
            .iter()
            .map(|(currency, value)| Amount::new(*value, currency))
    }
}

impl<'a> From<Amount<'a>> for Amounts<'a> {
    fn from(amount: Amount<'a>) -> Self {
        Amounts::default() + amount
    }
}

impl<'a> AddAssign<Amount<'a>> for Amounts<'a> {
    fn add_assign(&mut self, rhs: Amount<'a>) {
        let value = rhs.value().0;
        let _ = self
            .0
            .entry(rhs.currency)
            .and_modify(|v| *v += value)
            .or_insert(value);
    }
}

impl<'a> Add<Amount<'a>> for Amounts<'a> {
    type Output = Self;

    fn add(mut self, rhs: Amount<'a>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> Add<Amount<'a>> for Amount<'a> {
    type Output = Amounts<'a>;

    fn add(self, rhs: Amount<'a>) -> Self::Output {
        Amounts::from(self) + rhs
    }
}

impl<'a> Sum<Amount<'a>> for Amounts<'a> {
    fn sum<I: Iterator<Item = Amount<'a>>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, value| acc + value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_same_currency() {
        let amounts: Amounts<'_> = Amount::new(1, "CHF") + Amount::new(2, "CHF");
        let amounts: Vec<Amount<'_>> = amounts.iter().collect();
        assert_eq!(&amounts[..], &[Amount::new(3, "CHF")]);
    }

    #[test]
    fn add_different_currency() {
        let amounts: Amounts<'_> = Amount::new(1, "CHF") + Amount::new(3, "EUR");
        assert_eq!(amounts.iter().count(), 2);
        assert!(amounts.iter().any(|a| a == Amount::new(1, "CHF")));
        assert!(amounts.iter().any(|a| a == Amount::new(3, "EUR")));
    }

    #[test]
    fn sum() {
        let sum: Amounts<'_> = [
            Amount::new(1, "CHF"),
            Amount::new(-3, "CHF"),
            Amount::new(2, "EUR"),
        ]
        .into_iter()
        .sum();

        assert_eq!(sum.iter().count(), 2);
        assert!(sum.iter().any(|a| a == Amount::new(-2, "CHF")));
        assert!(sum.iter().any(|a| a == Amount::new(2, "EUR")));
    }
}
