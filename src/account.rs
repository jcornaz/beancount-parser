use std::{
    borrow::Borrow,
    collections::HashSet,
    fmt::{Display, Formatter},
    str::FromStr,
    sync::Arc,
};

use nom::{
    branch::alt,
    bytes::{complete::tag, complete::take_while},
    character::complete::{char, satisfy, space0, space1},
    combinator::{all_consuming, cut, iterator, opt, recognize},
    multi::many1_count,
    sequence::{delimited, preceded},
    Finish,
};

use crate::{
    amount::{self, Amount, Currency},
    Decimal, Span,
};

use super::IResult;

/// Account
///
/// You may convert it into a string slice with [`Account::as_str`]
///
/// # Example
/// ```
/// use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = "2022-05-24 open Assets:Bank:Checking";
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Open(open) = &beancount.directives[0].content else { unreachable!() };
/// assert_eq!(open.account.as_str(), "Assets:Bank:Checking");
/// ```
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Account(Arc<str>);

impl Account {
    /// Returns underlying string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Account {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for Account {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl FromStr for Account {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let spanned = Span::new(input);
        match all_consuming(parse)(spanned).finish() {
            Ok((_, account)) => Ok(account),
            Err(err) => {
                println!("{err:?}");
                Err(Self::Err::new(input, spanned))
            }
        }
    }
}

/// Open account directive
///
/// # Example
/// ```
/// use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = "2022-05-24 open Assets:Bank:Checking    CHF";
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Open(open) = &beancount.directives[0].content else { unreachable!() };
/// assert_eq!(open.account.as_str(), "Assets:Bank:Checking");
/// assert_eq!(open.currencies.iter().next().unwrap().as_str(), "CHF");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Open {
    /// Account being open
    pub account: Account,
    /// Currency constraints
    pub currencies: HashSet<Currency>,
    /// Booking method
    pub booking_method: Option<BookingMethod>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BookingMethod(Arc<str>);

impl AsRef<str> for BookingMethod {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for BookingMethod {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl Display for BookingMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<&str> for BookingMethod {
    fn from(value: &str) -> Self {
        Self(Arc::from(value))
    }
}

/// Close account directive
///
/// # Example
/// ```
/// use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = "2022-05-24 close Assets:Bank:Checking";
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Close(close) = &beancount.directives[0].content else { unreachable!() };
/// assert_eq!(close.account.as_str(), "Assets:Bank:Checking");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Close {
    /// Account being closed
    pub account: Account,
}

/// Balance assertion
///
/// # Example
/// ```
/// use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = "2022-05-24 balance Assets:Bank:Checking 10 CHF";
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Balance(balance) = &beancount.directives[0].content else { unreachable!() };
/// assert_eq!(balance.account.as_str(), "Assets:Bank:Checking");
/// assert_eq!(balance.amount.value, 10.0);
/// assert_eq!(balance.amount.currency.as_str(), "CHF");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Balance<D> {
    /// Account being asserted
    pub account: Account,
    /// Amount the amount should have on the date
    pub amount: Amount<D>,
    /// Explicit precision tolerance
    ///
    /// See: <https://beancount.github.io/docs/precision_tolerances.html#explicit-tolerances-on-balance-assertions>
    pub tolerance: Option<D>,
}

/// Pad directive
///
/// # Example
/// ```
/// # use beancount_parser::{BeancountFile, DirectiveContent};
/// let raw = "2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances";
/// let file: BeancountFile<f64> = raw.parse().unwrap();
/// let DirectiveContent::Pad(pad) = &file.directives[0].content else { unreachable!() };
/// assert_eq!(pad.account.as_str(), "Assets:BofA:Checking");
/// assert_eq!(pad.source_account.as_str(), "Equity:Opening-Balances");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Pad {
    /// Account being padded
    pub account: Account,
    /// Source account from which take the money
    pub source_account: Account,
}

pub(super) fn parse(input: Span<'_>) -> IResult<'_, Account> {
    let (input, name) = recognize(preceded(
        alt((
            tag("Expenses"),
            tag("Assets"),
            tag("Liabilities"),
            tag("Income"),
            tag("Equity"),
        )),
        cut(many1_count(preceded(
            char(':'),
            preceded(
                satisfy(|c: char| c.is_uppercase() || c.is_ascii_digit()),
                take_while(|c: char| c.is_alphanumeric() || c == '-'),
            ),
        ))),
    ))(input)?;
    Ok((input, Account(Arc::from(*name.fragment()))))
}

pub(super) fn open(input: Span<'_>) -> IResult<'_, Open> {
    let (input, account) = parse(input)?;
    let (input, currencies) = opt(preceded(space1, currencies))(input)?;
    let (input, booking_method) = opt(preceded(space1, crate::string))(input)?;
    Ok((
        input,
        Open {
            account,
            currencies: currencies.unwrap_or_default(),
            booking_method: booking_method.map(|s| s.as_str().into()),
        },
    ))
}

fn currencies(input: Span<'_>) -> IResult<'_, HashSet<Currency>> {
    let (input, first) = amount::currency(input)?;
    let sep = delimited(space0, char(','), space0);
    let mut iter = iterator(input, preceded(sep, amount::currency));
    let mut currencies = HashSet::new();
    currencies.insert(first);
    currencies.extend(&mut iter);
    let (input, ()) = iter.finish()?;
    Ok((input, currencies))
}

pub(super) fn close(input: Span<'_>) -> IResult<'_, Close> {
    let (input, account) = parse(input)?;
    Ok((input, Close { account }))
}

pub(super) fn balance<D: Decimal>(input: Span<'_>) -> IResult<'_, Balance<D>> {
    let (input, account) = parse(input)?;
    let (input, _) = space1(input)?;
    let (input, value) = amount::expression(input)?;
    let (input, tolerance) = opt(preceded(space0, tolerance))(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = amount::currency(input)?;
    Ok((
        input,
        Balance {
            account,
            amount: Amount { value, currency },
            tolerance,
        },
    ))
}

fn tolerance<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, _) = char('~')(input)?;
    let (input, _) = space0(input)?;
    let (input, tolerance) = amount::expression(input)?;
    Ok((input, tolerance))
}

pub(super) fn pad(input: Span<'_>) -> IResult<'_, Pad> {
    let (input, account) = parse(input)?;
    let (input, _) = space1(input)?;
    let (input, source_account) = parse(input)?;
    Ok((
        input,
        Pad {
            account,
            source_account,
        },
    ))
}

#[cfg(test)]
mod chumksy {
    use crate::ChumskyParser;

    use super::Account;
    use chumsky::prelude::*;

    fn account() -> impl ChumskyParser<Account> {
        let category = choice((
            just("Assets"),
            just("Liabilities"),
            just("Equity"),
            just("Income"),
            just("Expenses"),
        ));
        let component = filter(|c: &char| c.is_alphanumeric() || *c == '-')
            .repeated()
            .at_least(1);
        category
            .map(ToOwned::to_owned)
            .then(just(':').ignore_then(component).repeated().at_least(1))
            .foldl(|mut account, component| {
                account.push(':');
                account.extend(component);
                account
            })
            .map(|s| Account(s.into()))
            .labelled("account")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rstest::rstest;

        #[rstest]
        #[case::assets("Assets:A")]
        #[case::liabilities("Liabilities:A")]
        #[case::equity("Equity:A")]
        #[case::expenses("Expenses:A")]
        #[case::income("Income:A")]
        #[case::one_component("Assets:Cash")]
        #[case::multiple_components("Assets:Cash:Wallet")]
        #[case::dash("Assets:Hello-world")]
        #[case::num_at_end("Assets:Cash2")]
        #[case::num_at_start("Assets:2Cash")]
        fn should_parse_valid_account(#[case] input: &str) {
            let account: Account = account().then_ignore(end()).parse(input).unwrap();
            assert_eq!(account.as_str(), input);
        }

        #[rstest]
        #[case("Hello")]
        #[case("Assets")]
        #[case("Assets:")]
        #[case("Assets::A")]
        fn should_not_parse_invalid_account(#[case] input: &str) {
            let result = account().then_ignore(end()).parse(input);
            assert!(result.is_err(), "{result:?}");
        }
    }
}
