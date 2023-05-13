use crate::{transaction, Directive, Error, IResult, Span};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::{line_ending, not_line_ending},
        streaming::space1,
    },
    combinator::{map, opt, value},
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
        value(Chunk::Comment, tuple((not_line_ending, opt(line_ending)))),
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
fn parse(
    input: &str,
) -> Result<Vec<crate::span::Spanned<Directive<'_>>>, crate::span::Spanned<Error>> {
    use crate::span::Spanned;
    use nom::{multi::many0, Parser};
    match many0(directive.map(Spanned::new))(Span::new(input)) {
        Ok((_, directives)) => Ok(directives),
        Err(_) => Err(Spanned::new(Error::from_parsing())),
    }
}

#[cfg(all(test, feature = "unstable"))]
mod acceptance_tests {
    //! Tests destined to be moved out to the 'tests' directory when the new api will be made public.

    use rust_decimal::Decimal;

    use crate::amount::Expression;
    use crate::transaction::posting::LotAttributes;
    use crate::transaction::{Flag, Posting, PriceType};
    use crate::{account, Amount, Date};

    use super::*;

    const COMMENTS: &str = include_str!("../tests/samples/comments.beancount");
    const SIMPLE: &str = include_str!("../tests/samples/simple.beancount");
    const OFFICIAL: &str = include_str!("../tests/samples/official.beancount");

    #[rstest]
    #[ignore = "not implemented"]
    fn successful_parse(
        #[values("", " ", " \n ", " \t ", COMMENTS, SIMPLE, OFFICIAL)] input: &str,
    ) {
        if let Err(err) = parse(input) {
            panic!("{err:?}");
        }
    }

    #[rstest]
    fn comments(
        #[values(
            "",
            "\n",
            "\n\n\r\n",
            "2016 - 11 - 28 close Liabilities:CreditCard:CapitalOne",
            "Hello world",
            "* Banking",
            "** Bank of America",
            ";; Transactions follow …",
            "; foo bar",
            ";"
        )]
        input: &str,
    ) {
        let len = parse(input).expect("should successfully parse input").len();
        assert_eq!(len, 0);
    }

    #[rstest]
    #[case(
        "2016-11-28 close Liabilities:CreditCard:CapitalOne",
        Date::new(2016, 11, 28)
    )]
    #[case("2022-12-31 open Assets:A", Date::new(2022, 12, 31))]
    #[case("2000-01-01 txn", Date::new(2000, 1, 1))]
    #[case("2000-01-02 * \"Groceries\"", Date::new(2000, 1, 2))]
    #[case("2000-01-03 * \"Store\" \"Groceries\"", Date::new(2000, 1, 3))]
    #[case("2000-01-04 *", Date::new(2000, 1, 4))]
    #[case("2000-01-05 !", Date::new(2000, 1, 5))]
    #[case(
        "2020-01-02 balance Assets:US:BofA:Checking        3467.65 USD",
        Date::new(2020, 1, 2)
    )]
    fn parse_date(#[case] input: &str, #[case] expected: Date) {
        let directive = parse_single_directive(input);
        assert_eq!(directive.date(), Some(expected));
    }

    #[rstest]
    #[case("2000-01-01 txn", None)]
    #[case("2000-01-01 txn \"Store\"", None)]
    #[case("2000-01-01 *", Some(Flag::Cleared))]
    #[case("2000-01-01 * \"Store\"", Some(Flag::Cleared))]
    #[case("2000-01-01 !", Some(Flag::Pending))]
    #[case("2000-01-01 ! \"Store\"", Some(Flag::Pending))]
    fn parse_transaction_flag(#[case] input: &str, #[case] expected: Option<Flag>) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        assert_eq!(transaction.flag(), expected);
    }

    #[rstest]
    #[case("2022-02-12 txn", None, None)]
    #[case("2022-02-12  txn", None, None)]
    #[case("2022-02-12 *", None, None)]
    #[case("2022-02-12  *  ", None, None)]
    #[case("2022-02-12 txn \"Hello\"", None, Some("Hello"))]
    #[case("2022-02-12   txn  \"Hello\"", None, Some("Hello"))]
    #[case("2022-02-12 * \"Hello\"", None, Some("Hello"))]
    #[case("2022-02-12 txn \"Hello\" \"World\"", Some("Hello"), Some("World"))]
    #[case("2022-02-12 txn \"Hello\" \t \"World\"", Some("Hello"), Some("World"))]
    #[case("2022-02-12 ! \"Hello\" \"World\"", Some("Hello"), Some("World"))]
    fn parse_transaction_payee_and_description(
        #[case] input: &str,
        #[case] expected_payee: Option<&str>,
        #[case] expected_narration: Option<&str>,
    ) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        assert_eq!(transaction.payee(), expected_payee);
        assert_eq!(transaction.narration(), expected_narration);
    }

    #[rstest]
    #[case(r#"2022-02-12 txn"#, &[])]
    #[case(r#"2022-02-12 txn #hello"#, &["hello"])]
    #[case(r#"2022-02-12 txn "Payee" "Narration" #hello"#, &["hello"])]
    #[case(r#"2022-02-12 txn "Payee" "Narration" #Hello #world"#, &["Hello", "world"])]
    #[case(r#"2020-11-24 * "Legal Seafood" "" #trip-boston-2020"#, &["trip-boston-2020"])]
    fn parse_transaction_tags(#[case] input: &str, #[case] expected: &[&str]) {
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        assert_eq!(transaction.tags(), expected);
    }

    #[rstest]
    #[case("2022-02-12 txn", &[])]
    #[case("2022-02-12 txn\n  Assets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
    #[case("2022-02-12 txn ; Hello\n  Assets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
    #[case("2022-02-12 txn; Hello\n  Assets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
    #[case("2022-02-12 txn ; Hello\nAssets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
    #[case("2022-02-12 txn\nAssets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
    #[case("2020-11-24 * \"Legal Seafood\" \"\" #trip-boston-2020\n  Liabilities:US:Chase:Slate  -40.15 USD\n  Expenses:Food:Restaurant  40.15 USD", &["Liabilities:US:Chase:Slate", "Expenses:Food:Restaurant"])]
    fn parse_posting_accounts(#[case] input: &str, #[case] expected: &[&str]) {
        let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();
        let transaction = parse_single_directive(input).into_transaction().unwrap();
        let actual: Vec<String> = transaction
            .postings()
            .iter()
            .map(Posting::account)
            .map(ToString::to_string)
            .collect();
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case("Assets:Hello", None)]
    #[case("  Assets:Hello", None)]
    #[case("* Assets:Hello", Some(Flag::Cleared))]
    #[case("  * Assets:Hello", Some(Flag::Cleared))]
    #[case("  *  Assets:Hello", Some(Flag::Cleared))]
    #[case("! Assets:Hello", Some(Flag::Pending))]
    #[case("  ! Assets:Hello", Some(Flag::Pending))]
    #[case("  !  Assets:Hello", Some(Flag::Pending))]
    #[ignore = "not implemented"]
    fn parse_posting_flag(#[case] input: &str, #[case] expected: Option<Flag>) {
        let input = format!("2022-02-23 txn\n{input}");
        let transaction = parse_single_directive(&input).into_transaction().unwrap();
        let posting = &transaction.postings()[0];
        assert_eq!(posting.flag(), expected);
    }

    #[rstest]
    #[case("Assets:Hello", None)]
    #[case("Assets:Hello ; Hello", Some("Hello"))]
    #[case("Assets:Hello 10 CHF ; World", Some("World"))]
    #[case("Assets:Hello  10 CHF ; World", Some("World"))]
    #[case("Assets:Hello 10 CHF ;;;  World", Some("World"))]
    #[case("Assets:Hello 10 CHF; Tadaa", Some("Tadaa"))]
    #[ignore = "not implemented"]
    fn parse_posting_comment(#[case] input: &str, #[case] expected: Option<&str>) {
        let input = format!("2022-02-23 txn\n{input}");
        let transaction = parse_single_directive(&input).into_transaction().unwrap();
        let posting = &transaction.postings()[0];
        assert_eq!(posting.comment(), expected);
    }

    #[rstest]
    #[case("2022-02-12 txn\n  Assets:Hello", None)]
    #[case("2022-02-12  txn \nAssets:Hello", None)]
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
    #[ignore = "not implemented"]
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
    #[ignore = "not implemented"]
    #[case(
        "2023-02-27 txn\n  Assets:A 10 CHF@19 EUR",
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
    #[case("2", Expression::value(2))]
    #[case("2+3", Expression::plus(Expression::value(2), Expression::value(3)))]
    #[case("2 + 3", Expression::plus(Expression::value(2), Expression::value(3)))]
    #[case(
        "2 + 3 + 4",
        Expression::plus(
            Expression::plus(Expression::value(2), Expression::value(3)),
            Expression::value(4)
        )
    )]
    #[case(
        "2 + 3 - 4",
        Expression::minus(
            Expression::plus(Expression::value(2), Expression::value(3)),
            Expression::value(4)
        )
    )]
    #[case("2*3", Expression::mul(Expression::value(2), Expression::value(3)))]
    #[case("2 * 3", Expression::mul(Expression::value(2), Expression::value(3)))]
    #[case("2  *  3", Expression::mul(Expression::value(2), Expression::value(3)))]
    #[case("2 / 3", Expression::div(Expression::value(2), Expression::value(3)))]
    #[case(
        "2  / \t3",
        Expression::div(Expression::value(2), Expression::value(3))
    )]
    #[case(
        "1 + 2 * 3",
        Expression::plus(
            Expression::value(1),
            Expression::mul(Expression::value(2), Expression::value(3)),
        )
    )]
    #[case(
        "1+2/3",
        Expression::plus(
            Expression::value(1),
            Expression::div(Expression::value(2), Expression::value(3)),
        )
    )]
    #[case(
        "(1+2)/3",
        Expression::div(
            Expression::plus(Expression::value(1), Expression::value(2)),
            Expression::value(3),
        )
    )]
    #[case(
        "( 1 + 2 ) / 3",
        Expression::div(
            Expression::plus(Expression::value(1), Expression::value(2)),
            Expression::value(3),
        )
    )]
    #[case(
        "(3 - 2) / 1",
        Expression::div(
            Expression::minus(Expression::value(3), Expression::value(2)),
            Expression::value(1),
        )
    )]
    #[case(
        "(3 - (2 * 1))",
        Expression::minus(
            Expression::value(3),
            Expression::mul(Expression::value(2), Expression::value(1)),
        )
    )]
    #[case(
        "((2 * 1) - 3)",
        Expression::minus(
            Expression::mul(Expression::value(2), Expression::value(1)),
            Expression::value(3),
        )
    )]
    #[case(
        "3+4   *5/(  6* 2  )  --71",
        Expression::minus(
            Expression::plus(
                Expression::value(3),
                Expression::div(
                    Expression::mul(Expression::value(4), Expression::value(5)),
                    Expression::mul(Expression::value(6), Expression::value(2))
                )
            ),
            Expression::value(-71),
        )
    )]
    fn parse_expression(#[case] input: &str, #[case] expected: Expression) {
        let input = format!("2022-02-20 txn\n  Assets:A  {input} CHF");
        let transaction = parse_single_directive(&input).into_transaction().unwrap();
        let actual = transaction.postings()[0]
            .amount()
            .expect("no amount")
            .expression();
        assert_eq!(actual, &expected);
    }

    #[rstest]
    #[case("2016-11-28 close Assets:Hello", account::Type::Assets)]
    #[case("2016-11-28 close Liabilities:Hello", account::Type::Liabilities)]
    #[case("2016-11-28 close Expenses:Hello", account::Type::Expenses)]
    #[case("2016-11-28 close Income:Hello", account::Type::Income)]
    #[case("2016-11-28 close Equity:Hello", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello ; Foo bar", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello; Foo bar", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello;Foo bar", account::Type::Equity)]
    #[case("2016-11-28 close Equity:Hello;", account::Type::Equity)]
    #[case("2016-11-28  close  Equity:Hello;", account::Type::Equity)]
    #[case("2016-11-28\tclose\tEquity:Hello;", account::Type::Equity)]
    fn parse_close_directive_account_type(
        #[case] input: &str,
        #[case] expected_account_type: account::Type,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
        assert_eq!(close.account().type_(), expected_account_type);
    }

    #[rstest]
    #[case("2016-11-28 open Assets:Hello", account::Type::Assets)]
    #[case("2016-11-28  open  Assets:Hello", account::Type::Assets)]
    #[case("2016-11-28 open Liabilities:Hello", account::Type::Liabilities)]
    fn parse_open_directive_account_type(
        #[case] input: &str,
        #[case] expected_account_type: account::Type,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
        assert_eq!(open.account().type_(), expected_account_type);
    }

    #[rstest]
    #[case("2016-11-28 close Liabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    #[case("2016-11-28 close Assets:Hello", &["Hello"])]
    #[case("2016-11-28 close Assets", &[])]
    #[case("2016-11-28 close Assets:Hello-World:123", &["Hello-World", "123"])]
    #[case("2016-11-28  close\tLiabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    fn parse_close_directive_account_components(
        #[case] input: &str,
        #[case] expected_account_components: &[&str],
    ) {
        let directive = parse_single_directive(input);
        let Directive::Close(close) = directive else { panic!("expected close directive but was {directive:?}") };
        assert_eq!(close.account().components(), expected_account_components);
    }

    #[rstest]
    #[case("2016-11-28 open Liabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    #[case("2016-11-28 open Assets:Hello", &["Hello"])]
    #[case("2016-11-28 open Assets", &[])]
    #[case("2016-11-28 open Assets:Hello-World:123", &["Hello-World", "123"])]
    #[case("2016-11-28  open\t\tLiabilities:CreditCard:CapitalOne", &["CreditCard", "CapitalOne"])]
    fn parse_open_directive_account_components(
        #[case] input: &str,
        #[case] expected_account_components: &[&str],
    ) {
        let directive = parse_single_directive(input);
        let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
        assert_eq!(open.account().components(), expected_account_components);
    }

    #[rstest]
    #[case("2016-11-28 open Assets", &[])]
    #[case("2016-11-28 open Assets CHF", &["CHF"])]
    #[case("2016-11-28  open  Assets CHF", &["CHF"])]
    #[case("2016-11-28\topen\tAssets CHF", &["CHF"])]
    #[case("2016-11-28 open Assets CHF,EUR", &["CHF", "EUR"])]
    #[case("2016-11-28 open Assets CHF , EUR", &["CHF", "EUR"])]
    #[case("2016-11-28 open Assets AB-CD, A_2B, A.B, A'B", &["AB-CD", "A_2B", "A.B", "A'B"])]
    #[ignore = "not implemented"]
    fn parse_open_directive_currencies(#[case] input: &str, #[case] expected: &[&str]) {
        let directive = parse_single_directive(input);
        let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
        assert_eq!(open.currencies(), expected);
    }

    #[rstest]
    #[ignore = "not implemented"]
    fn parse_commodity_currency(
        #[values(
            "1792-01-01 commodity USD",
            "1792-01-01  commodity  USD",
            "1792-01-01\tcommodity\tUSD"
        )]
        input: &str,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Commodity(commodity) = directive else { panic!("expected commodity but was {directive:?}") };
        assert_eq!(commodity.currency(), "USD");
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

    #[rstest]
    #[ignore = "not implemented"]
    fn parse_option(
        #[values(
            r#"option "operating_currency" "USD""#,
            r#"option  "operating_currency"  "USD""#,
            "option\t\"operating_currency\"\t\"USD\""
        )]
        input: &str,
    ) {
        let directive = parse_single_directive(input);
        let Directive::Option(option) = directive else { panic!("expected option but was {directive:?}") };
        assert_eq!(option.name(), "operating_currency");
        assert_eq!(option.value(), "USD");
    }

    #[test]
    #[ignore = "not implemented"]
    fn parse_event() {
        let input = r#"2020-11-23  event  "location"  "Boston""#;
        let directive = parse_single_directive(input);
        let Directive::Event(event) = directive else { panic!("expected event but was {directive:?}") };
        assert_eq!(event.date(), Date::new(2020, 11, 23));
        assert_eq!(event.name(), "location");
        assert_eq!(event.value(), "Boston");
    }

    #[rstest]
    #[ignore = "not implemented"]
    fn error_case(
        #[values(
            "2016-11-28closeLiabilities:CreditCard:CapitalOne",
            "2016-11-28 closeLiabilities:CreditCard:CapitalOne",
            "2016-11-28close Liabilities:CreditCard:CapitalOne",
            "2016-11-28 close Liabilities:CreditCard:CapitalOne Oops",
            "2016-11-28 close Oops",
            "2016-11-28openAssets:A",
            "2016-11-28 openAssets:A",
            "2016-11-28open Assets:A",
            "2016-11-28 open Assets:A oops",
            "2016-11-28 open Assets:A 22",
            "2016-11-28 open Oops",
            "2020-01-03price VBMPX 186 USD",
            "2020-01-03 priceVBMPX 186 USD",
            "2020-01-03 price VBMPX186 USD",
            "2020-01-03 price VBMPX",
            "2020-01-03 price ",
            "2022-02-12 txn\n  Assets:Hello  10CHF",
            "2022-02-12 txn\n  Assets:Hello10 CHF",
            "2022-02-12 txn\n  Assets:Hello 1 +  CHF",
            "2022-02-12 txn\n  Assets:Hello 2 *  CHF",
            "2022-02-12 txn\n  Assets:Hello 1 /  CHF",
            "2022-02-12 txn\n  Assets:Hello 1 CHF {",
            "2022-02-12 txn\n  Assets:Hello 1 CHF }",
            "2022-02-12 txn\n  Assets:Hello 1 CHF {,}",
            "2022-02-12 txn\n  Assets:Hello 1 CHF {1 CHF,}",
            "2022-02-12 txn\n  Assets:Hello 1 CHF {,1 CHF}",
            "2022-02-12 txn\n  Assets:Hello 1 CHF @",
            "2022-02-12 txn\n  Assets:Hello 1 CHF @ ",
            "2022-02-12 txn\n  Assets:Hello 1 CHF @@",
            "2022-02-12 txnAssets:Hello 1 /  CHF",
            "2022-02-12 txn oops",
            r#"2022-02-12 txn "hello""world""#,
            r#"2022-02-12 txn"hello" "world""#,
            r#"2022-02-12 txn#Hello"#,
            r#"2022-02-12 txn #Hello#world"#,
            r#"2022-02-12 txn #"#,
            r#"2022-02-12 txn ##"#,
            r#"2022-02-12 txn # #"#,
            "1792-01-01 commodityUSD",
            "1792-01-01commodity USD",
            "1792-01-01 balance",
            "1792-01-01 balance Test",
            "1792-01-01 balance Assets:A",
            "1792-01-01 balance Assets:A",
            "1792-01-01 balanceAssets:A 1 CHF",
            "1792-01-01balance Assets:A 1 CHF",
            r#"2020-11-23event "location" "Boston""#,
            r#"2020-11-23 event"location" "Boston""#,
            r#"2020-11-23 event "location""Boston""#,
            "option",
            "option a b",
            "option \"a\"",
            "option \"a\" b",
            "option\"a\" \"b\"",
            "option \"a\"\"b\""
        )]
        input: &str,
    ) {
        let result = parse(input);
        assert!(result.is_err(), "{result:?}");
    }

    fn parse_single_directive(input: &str) -> Directive<'_> {
        let directives = parse(input).expect("failed to parse input");
        assert_eq!(directives.len(), 1, "unexpected number of directives");
        directives.into_iter().next().unwrap().into_inner()
    }
}
