use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter},
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
    sync::Arc,
};

use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, one_of, satisfy, space0, space1},
    combinator::{all_consuming, iterator, map_res, opt, recognize, verify},
    sequence::{delimited, preceded, terminated, tuple},
    Finish,
};

use crate::{IResult, Span};

/// Price directive
///
/// # Example
///
/// ```
/// use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = "2023-05-27 price CHF  4 PLN";
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Price(price) = &beancount.directives[0].content else { unreachable!() };
/// assert_eq!(price.currency.as_str(), "CHF");
/// assert_eq!(price.amount.value, 4.0);
/// assert_eq!(price.amount.currency.as_str(), "PLN");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Price<D> {
    /// Currency
    pub currency: Currency,
    /// Price of the currency
    pub amount: Amount<D>,
}

/// Amount
///
/// Where `D` is the decimal type (like `f64` or `rust_decimal::Decimal`)
///
/// For an example, look at the [`Price`] directive
#[derive(Debug, Clone, PartialEq)]
pub struct Amount<D> {
    /// The value (decimal) part
    pub value: D,
    /// Currency
    pub currency: Currency,
}

/// Currency
///
/// One may use [`Currency::as_str`] to get the string representation of the currency
///
/// For an example, look at the [`Price`] directive
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Currency(Arc<str>);

impl Currency {
    /// Returns underlying string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Currency {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for Currency {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl<'a> TryFrom<&'a str> for Currency {
    type Error = crate::ConversionError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match all_consuming(currency)(Span::new(value)).finish() {
            Ok((_, currency)) => Ok(currency),
            Err(_) => Err(crate::ConversionError),
        }
    }
}

pub(crate) fn parse<D: Decimal>(input: Span<'_>) -> IResult<'_, Amount<D>> {
    let (input, value) = expression(input)?;
    let (input, _) = space1(input)?;
    let (input, currency) = currency(input)?;
    Ok((input, Amount { value, currency }))
}

pub(crate) fn expression<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    alt((negation, sum))(input)
}

fn sum<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, value) = product(input)?;
    let mut iter = iterator(
        input,
        tuple((delimited(space0, one_of("+-"), space0), product)),
    );
    let value = iter.fold(value, |a, (op, b)| match op {
        '+' => a + b,
        '-' => a - b,
        op => unreachable!("unsupported operator: {}", op),
    });
    let (input, ()) = iter.finish()?;
    Ok((input, value))
}

fn product<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, value) = atom(input)?;
    let mut iter = iterator(
        input,
        tuple((delimited(space0, one_of("*/"), space0), atom)),
    );
    let value = iter.fold(value, |a, (op, b)| match op {
        '*' => a * b,
        '/' => a / b,
        op => unreachable!("unsupported operator: {}", op),
    });
    let (input, ()) = iter.finish()?;
    Ok((input, value))
}

fn atom<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    alt((literal, group))(input)
}

fn group<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    delimited(
        terminated(char('('), space0),
        expression,
        preceded(space0, char(')')),
    )(input)
}

fn negation<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    let (input, _) = char('-')(input)?;
    let (input, _) = space0(input)?;
    let (input, expr) = group::<D>(input)?;
    Ok((input, -expr))
}

fn literal<D: Decimal>(input: Span<'_>) -> IResult<'_, D> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            space0,
            take_while1(|c: char| c.is_numeric() || c == '.' || c == ','),
        ))),
        |s: Span<'_>| s.fragment().replace([',', ' '], "").parse(),
    )(input)
}

pub(crate) fn price<D: Decimal>(input: Span<'_>) -> IResult<'_, Price<D>> {
    let (input, currency) = currency(input)?;
    let (input, _) = space1(input)?;
    let (input, amount) = parse(input)?;
    Ok((input, Price { currency, amount }))
}

pub(crate) fn currency(input: Span<'_>) -> IResult<'_, Currency> {
    let (input, currency) = recognize(tuple((
        satisfy(char::is_uppercase),
        verify(
            take_while(|c: char| {
                c.is_uppercase() || c.is_numeric() || c == '-' || c == '_' || c == '.' || c == '\''
            }),
            |s: &Span<'_>| {
                s.fragment()
                    .chars()
                    .last()
                    .map_or(true, |c| c.is_uppercase() || c.is_numeric())
            },
        ),
    )))(input)?;
    Ok((input, Currency(Arc::from(*currency.fragment()))))
}

/// Decimal type to which amount values and expressions will be parsed into.
///
/// # Notable implementations
///
/// * `f64`
/// * `Decimal` of the crate [rust_decimal]
///
/// [rust_decimal]: https://docs.rs/rust_decimal
///
pub trait Decimal:
    FromStr
    + Default
    + Clone
    + Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + PartialEq
    + PartialOrd
{
}

impl<D> Decimal for D where
    D: FromStr
        + Default
        + Clone
        + Debug
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + PartialEq
        + PartialOrd
{
}

#[cfg(test)]
pub(crate) mod chumsky {
    use super::{Amount, Currency};
    use crate::{ChumskyError, ChumskyParser, Decimal};

    use chumsky::{prelude::*, text::whitespace};

    pub(crate) fn amount<D: Decimal + 'static>() -> impl ChumskyParser<Amount<D>> {
        expression::<D>()
            .then_ignore(whitespace())
            .then(currency())
            .map(|(value, currency)| Amount { value, currency })
    }

    pub(crate) fn currency() -> impl ChumskyParser<Currency> {
        filter(|c: &char| c.is_ascii_uppercase())
            .chain(
                filter(|c: &char| c.is_ascii_uppercase() || c.is_ascii_digit())
                    .or(one_of("'.-_"))
                    .repeated(),
            )
            .collect::<String>()
            .map(|s| Currency(s.into()))
    }

    fn expression<D: Decimal + 'static>() -> impl ChumskyParser<D> {
        recursive(|expr| {
            let atom = atom(expr);
            let product = product(atom);
            sum(product)
        })
        .labelled("expression")
    }

    fn sum<D: Decimal>(atom: impl ChumskyParser<D> + Clone) -> impl ChumskyParser<D> + Clone {
        atom.clone()
            .then(just('+').or(just('-')).padded().then(atom).repeated())
            .foldl(|sum, (op, value)| match op {
                '+' => sum + value,
                '-' => sum - value,
                op => unreachable!("unknown sum operator: {op}"),
            })
    }

    fn product<D: Decimal>(atom: impl ChumskyParser<D> + Clone) -> impl ChumskyParser<D> + Clone {
        atom.clone()
            .then(just('*').or(just('/')).padded().then(atom).repeated())
            .foldl(|product, (op, value)| match op {
                '*' => product * value,
                '/' => product / value,
                op => unreachable!("unknown product operator: {op}"),
            })
    }

    fn atom<D: Decimal>(expr: impl ChumskyParser<D> + Clone) -> impl ChumskyParser<D> + Clone {
        just('-')
            .or_not()
            .then(
                expr.padded()
                    .delimited_by(just('('), just(')'))
                    .or(literal::<D>()),
            )
            .map(|(neg_op, b)| if neg_op == Some('-') { -b } else { b })
    }

    fn literal<D: Decimal>() -> impl ChumskyParser<D> + Copy {
        let digit = filter(|c: &char| c.is_ascii_digit());
        let int_part = digit.repeated().at_least(1).chain::<char, _, _>(
            just(',')
                .chain(digit.repeated().at_least(1))
                .repeated()
                .flatten(),
        );
        let fract_part = just('.').chain::<char, _, _>(digit.repeated()).or_not();
        int_part
            .chain::<char, _, _>(fract_part)
            .collect::<String>()
            .try_map(|string: String, span| {
                string
                    .replace(',', "")
                    .parse()
                    .map_err(|_| ChumskyError::custom(span, "not a number"))
            })
            .labelled("numeric value")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rstest::rstest;

        #[rstest]
        #[case::literal("42", 42)]
        #[case::neg_literal("-42", -42)]
        #[case::double_neg("-42", -42)]
        #[case::neg_parenthesis("-(42)", -42)]
        #[case::neg_parenthesis_2("-(-42)", 42)]
        #[case::addition("1+2+3", 6)]
        #[case::addition_with_space("1 + 2", 3)]
        #[case::substraction("10-2-5", 3)]
        #[case::substraction_with_space("5 - 3", 2)]
        #[case::multiplication("2*3*4", 24)]
        #[case::multiplication_with_space("2 * 3", 6)]
        #[case::division("6/3", 2)]
        #[case::division_with_space("6 / 3", 2)]
        #[case::operator_priority("1 + 2 * 3", 7)]
        #[case::parenthesis("(1+2)*3", 9)]
        #[case::parenthesis_nested("( 1 + ( 2 + 2 ) * 4 ) * 3", 51)]
        fn should_parse_valid_expression(#[case] input: &str, #[case] expected: i32) {
            let result: i32 = expression().then_ignore(end()).parse(input).unwrap();
            assert_eq!(result, expected);
        }

        #[rstest]
        #[case::zero("0", 0.)]
        #[case::zero_one("01", 1.)]
        #[case::zero_dot("0.", 0.)]
        #[case::int("42", 42.)]
        #[case::with_fract_part("42.42", 42.42)]
        #[case::thousand("1000", 1_000.)]
        #[case::thousand_sep("1,000", 1_000.)]
        fn should_parse_integer(#[case] input: &str, #[case] expected: f64) {
            let value: f64 = literal().then_ignore(end()).parse(input).unwrap();
            assert!(
                (value - expected).abs() <= f64::EPSILON,
                "{value} should equal {expected}"
            );
        }

        #[rstest]
        #[case::empty("")]
        #[case::alpha("x")]
        #[case::start_with_dot(".0")]
        #[case::start_with_thousand_sep(",1")]
        #[case::two_dots("1..")]
        #[case::comma_in_fract_part("1.2,3")]
        #[case::comma_dot("1,.0")]
        fn should_not_parse_invalid_value(#[case] input: &str) {
            let result = literal::<f64>().then_ignore(end()).parse(input);
            assert!(result.is_err(), "{result:?}");
        }

        #[rstest]
        #[case::normal("CHF")]
        #[case::with_special_chars("USD'42-CHF_EUR.PLN")]
        #[case::end_with_digit("A2")]
        fn should_parse_valid_currency(#[case] input: &str) {
            let currency: Currency = currency().parse(input).unwrap();
            assert_eq!(currency.as_str(), input);
        }

        #[rstest]
        #[case::empty("")]
        #[case::lowercase("chf")]
        #[case::start_with_digit("2A")]
        fn should_not_parse_invalid_currency(#[case] input: &str) {
            let result: Result<Currency, _> = currency().parse(input);
            assert!(result.is_err(), "{result:?}");
        }

        #[rstest]
        fn should_parse_valid_amount() {
            let amount: Amount<i32> = amount().parse("1 + 3 CHF").unwrap();
            assert_eq!(amount.value, 4);
            assert_eq!(amount.currency.as_str(), "CHF");
        }
    }
}
