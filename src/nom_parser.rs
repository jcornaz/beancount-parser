use crate::{transaction, Directive, Error, IResult, Span};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{line_ending, space1},
    combinator::{map, opt, recognize},
    sequence::{preceded, tuple},
};

use crate::directive::directive;

/// Parser of a beancount document
///
/// It is an iterator over the beancount directives.
///
/// See the crate documentation for usage example.
#[allow(missing_debug_implementations)]
pub struct Parser<'a> {
    rest: Span<'a>,
    tags: Vec<&'a str>,
    line: u64,
}

impl<'a> Parser<'a> {
    /// Create a new parser from the beancount string to parse
    #[must_use]
    pub fn new(content: &'a str) -> Self {
        Self {
            rest: Span::new(content),
            tags: Vec::new(),
            line: 1,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Directive<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.rest.is_empty() {
            if let Ok((rest, chunk)) = chunk(self.rest) {
                self.line += 1;
                self.rest = rest;
                match chunk {
                    Chunk::Directive(mut directive) => {
                        if let Directive::Transaction(trx) = &mut directive {
                            self.line += trx.postings().len() as u64;
                            trx.append_tags(&self.tags);
                        }
                        return Some(Ok(directive));
                    }
                    Chunk::PushTag(tag) => self.tags.push(tag),
                    Chunk::PopTag(tag) => self.tags.retain(|&t| t != tag),
                    Chunk::Comment => (),
                }
            } else {
                self.rest = Span::new("");
                return Some(Err(Error::from_parsing()));
            }
        }
        None
    }
}

fn chunk(input: Span<'_>) -> IResult<'_, Chunk<'_>> {
    alt((
        map(directive, Chunk::Directive),
        map(pushtag, Chunk::PushTag),
        map(poptag, Chunk::PopTag),
        map(
            alt((
                recognize(tuple((take_till1(|c| c == '\n'), opt(line_ending)))),
                line_ending,
            )),
            |_| Chunk::Comment,
        ),
    ))(input)
}

fn pushtag(input: Span<'_>) -> IResult<'_, &str> {
    preceded(tuple((tag("pushtag"), space1)), transaction::tag)(input)
}

fn poptag(input: Span<'_>) -> IResult<'_, &str> {
    preceded(tuple((tag("poptag"), space1)), transaction::tag)(input)
}

#[derive(Debug, Clone)]
enum Chunk<'a> {
    Directive(Directive<'a>),
    Comment,
    PushTag(&'a str),
    PopTag(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushtag() {
        let input = "pushtag #test";
        let (_, chunk) = chunk(Span::new(input)).expect("should successfully parse the input");
        assert!(matches!(chunk, Chunk::PushTag("test")));
    }

    #[test]
    fn poptag() {
        let input = "poptag #test";
        let (_, chunk) = chunk(Span::new(input)).expect("should successfully parse the input");
        assert!(matches!(chunk, Chunk::PopTag("test")));
    }
}

#[cfg(all(test, feature = "unstable"))]
mod acceptance_tests {
    //! Tests destined to be moved out to the 'tests' directory when the new api will be made public.

    use rust_decimal::Decimal;

    use crate::transaction::posting::LotAttributes;
    use crate::transaction::PriceType;
    use crate::{Amount, Date};

    use super::*;

    #[rstest]
    #[case("2022-02-12 txn\n  Assets:Hello", None)]
    #[case("2022-02-12  txn \n  Assets:Hello", None)]
    #[case("2022-02-12 txn\n  Assets:Hello  10 CHF", Some(Amount::new(10, "CHF")))]
    #[case(
        "2022-02-12 txn\n  Assets:Hello  10  \tCHF",
        Some(Amount::new(10, "CHF"))
    )]
    #[case("2022-02-12 txn\n  Assets:Hello  -2 CHF", Some(Amount::new(-2, "CHF")))]
    #[case("2022-02-12 txn\n  Assets:Hello  +2 CHF", Some(Amount::new(2, "CHF")))]
    #[case(
        "2022-02-12 txn\n  Assets:Hello  1.2 CHF",
        Some(Amount::new(Decimal::new(12, 1), "CHF"))
    )]
    #[case(
        "2022-02-12 txn\n  Assets:Hello  0.2 CHF",
        Some(Amount::new(Decimal::new(2, 1), "CHF"))
    )]
    #[case(
        "2022-02-12 txn\n  Assets:Hello  .2 CHF",
        Some(Amount::new(Decimal::new(2, 1), "CHF"))
    )]
    fn parse_posting_amount(#[case] input: &str, #[case] expected: Option<Amount<'_>>) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        let posting = &transaction.postings()[0];
        assert_eq!(posting.amount(), expected.as_ref());
    }

    #[rstest]
    #[case("2023-02-27 txn\n  Assets:A 10 CHF", None)]
    #[case(
        "2023-02-27 txn\n  Assets:A 10 CHF {40 PLN}",
        Some(Amount::new(40, "PLN"))
    )]
    #[case(
        "2023-02-27 txn\n  Assets:A 10 CHF {  40  PLN  }",
        Some(Amount::new(40, "PLN"))
    )]
    #[case("2023-02-27 txn\n  Assets:A 10 CHF {}", None)]
    #[case("2023-02-27 txn\n  Assets:A 10 CHF { }", None)]
    #[case("2023-02-27 txn\n  Assets:A 10 CHF {\t}", None)]
    #[case(
        "2023-02-27 txn\n  Assets:A  10 CHF {42 PLN,2001-02-03}",
        Some(Amount::new(42, "PLN"))
    )]
    fn parse_posting_cost(#[case] input: &str, #[case] expected: Option<Amount<'_>>) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        let posting = &transaction.postings()[0];
        assert_eq!(posting.cost(), expected.as_ref());
        assert_eq!(
            posting.lot().and_then(LotAttributes::cost),
            expected.as_ref()
        );
    }

    #[rstest]
    #[case("2023-02-27 txn\n  Assets:A  10 CHF", None)]
    #[case("2023-02-27 txn\n  Assets:A  10 CHF {}", None)]
    #[case("2023-02-27 txn\n  Assets:A  10 CHF {40 PLN}", None)]
    #[case(
        "2023-02-27 txn\n  Assets:A  10 CHF {2000-01-02}",
        Some(Date::new(2000, 1, 2))
    )]
    #[case(
        "2023-02-27 txn\n  Assets:A  10 CHF {40 PLN,2001-02-03}",
        Some(Date::new(2001, 2, 3))
    )]
    #[case(
        "2023-02-27 txn\n  Assets:A  10 CHF {40 PLN, 2001-02-03}",
        Some(Date::new(2001, 2, 3))
    )]
    #[case(
        "2023-02-27 txn\n  Assets:A  10 CHF {  40 PLN  , 2001-02-03  }",
        Some(Date::new(2001, 2, 3))
    )]
    fn parse_posting_lot_date(#[case] input: &str, #[case] expected: Option<Date>) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        let posting = &transaction.postings()[0];
        assert_eq!(posting.lot().and_then(LotAttributes::date), expected);
    }

    #[rstest]
    #[case("2023-02-27 txn\n  Assets:A 10 CHF", None)]
    #[case(
        "2023-02-27 txn\n  Assets:A 10 CHF @ 19 EUR",
        Some((PriceType::Unit, Amount::new(19, "EUR")))
    )]
    #[case(
        "2023-02-27 txn\n  Assets:A 10 CHF  @  2 EUR",
        Some((PriceType::Unit, Amount::new(2, "EUR")))
    )]
    #[case(
        "2023-02-27 txn\n  Assets:A 10 CHF  @@  20 EUR",
        Some((PriceType::Total, Amount::new(20, "EUR")))
    )]
    fn parse_posting_price(#[case] input: &str, #[case] expected: Option<(PriceType, Amount<'_>)>) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        let posting = &transaction.postings()[0];
        assert_eq!(posting.price().map(|(t, a)| (t, a.clone())), expected);
    }

    #[rstest]
    fn parse_price(
        #[values(
            "2020-01-03 price VBMPX 186 USD",
            "2020-01-03  price  VBMPX  186  USD",
            "2020-01-03\tprice\tVBMPX\t186\tUSD"
        )]
        input: &str,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Price(price) = directive else { panic!("expected price but was {directive:?}") };
        assert_eq!(price.date(), Date::new(2020, 1, 3));
        assert_eq!(price.commodity(), "VBMPX");
        assert_eq!(price.price(), &Amount::new(186, "USD"));
    }

    #[rstest]
    fn parse_balance_assertion(
        #[values(
            "2020-01-02 balance Assets:US:BofA:Checking 1.2 USD",
            "2020-01-02  balance  Assets:US:BofA:Checking  1.2  USD",
            "2020-01-02\tbalance\tAssets:US:BofA:Checking\t1.2\tUSD"
        )]
        input: &str,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Assertion(assertion) = directive else { panic!("expected balance assertion but was {directive:?}") };
        assert_eq!(assertion.date(), Date::new(2020, 1, 2));
        assert_eq!(&assertion.account().to_string(), "Assets:US:BofA:Checking");
        assert_eq!(assertion.amount(), &Amount::new(Decimal::new(12, 1), "USD"));
    }

    fn parse_single_directive(input: &str) -> Directive<'_> {
        let directives = Parser::new(input)
            .collect::<Result<Vec<_>, _>>()
            .expect("failed to parse input");
        assert_eq!(directives.len(), 1, "unexpected number of directives");
        directives.into_iter().next().unwrap()
    }
}
