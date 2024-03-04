use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::satisfy,
    character::complete::{char as char_tag, space0, space1},
    combinator::{cut, iterator, map, opt, success, value},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Parser,
};

use crate::string;
use crate::{
    account, account::Account, amount, amount::Amount, date, empty_line, end_of_line, metadata,
    Date, Decimal, IResult, Span,
};

/// A transaction
///
/// It notably contains a list of [`Posting`]
///
/// # Example
/// ```
/// # use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = r#"
/// 2022-05-22 * "Grocery store" "Grocery shopping" #food
///   Assets:Cash           -10 CHF
///   Expenses:Groceries
/// "#;
///
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Transaction(trx) = &beancount.directives[0].content else {
///   unreachable!("was not a transaction")
/// };
/// assert_eq!(trx.flag, Some('*'));
/// assert_eq!(trx.payee.as_deref(), Some("Grocery store"));
/// assert_eq!(trx.narration.as_deref(), Some("Grocery shopping"));
/// assert!(trx.tags.contains("food"));
/// assert_eq!(trx.postings.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Transaction<D> {
    /// Transaction flag (`*` or `!` or `None` when using the `txn` keyword)
    pub flag: Option<char>,
    /// Payee (if present)
    pub payee: Option<String>,
    /// Narration (if present)
    pub narration: Option<String>,
    /// Set of tags
    pub tags: HashSet<Tag>,
    /// Set of links
    pub links: HashSet<Link>,
    /// Postings
    pub postings: Vec<Posting<D>>,
}

/// A transaction posting
///
/// # Example
/// ```
/// # use beancount_parser::{BeancountFile, DirectiveContent, PostingPrice};
/// let input = r#"
/// 2022-05-22 * "Grocery shopping"
///   Assets:Cash           1 CHF {2 PLN} @ 3 EUR
///   Expenses:Groceries
/// "#;
///
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Transaction(trx) = &beancount.directives[0].content else {
///   unreachable!("was not a transaction")
/// };
/// let posting = &trx.postings[0];
/// assert_eq!(posting.account.as_str(), "Assets:Cash");
/// assert_eq!(posting.amount.as_ref().unwrap().value, 1.0);
/// assert_eq!(posting.amount.as_ref().unwrap().currency.as_str(), "CHF");
/// assert_eq!(posting.cost.as_ref().unwrap().amount.as_ref().unwrap().value, 2.0);
/// assert_eq!(posting.cost.as_ref().unwrap().amount.as_ref().unwrap().currency.as_str(), "PLN");
/// let Some(PostingPrice::Unit(price)) = &posting.price else {
///   unreachable!("no price");
/// };
/// assert_eq!(price.value, 3.0);
/// assert_eq!(price.currency.as_str(), "EUR");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Posting<D> {
    /// Transaction flag (`*` or `!` or `None` when absent)
    pub flag: Option<char>,
    /// Account modified by the posting
    pub account: Account,
    /// Amount being added to the account
    pub amount: Option<Amount<D>>,
    /// Cost (content within `{` and `}`)
    pub cost: Option<Cost<D>>,
    /// Price (`@` or `@@`) syntax
    pub price: Option<PostingPrice<D>>,
    /// The metadata attached to the posting
    pub metadata: metadata::Map<D>,
}

/// Cost of a posting
///
/// It is the amount within `{` and `}`.
#[derive(Debug, Default, Clone, PartialEq)]
#[non_exhaustive]
pub struct Cost<D> {
    /// Cost basis of the posting
    pub amount: Option<Amount<D>>,
    /// The date of this cost basis
    pub date: Option<Date>,
}

/// Price of a posting
///
/// It is the amount following the `@` or `@@` symbols
#[derive(Debug, Clone, PartialEq)]
pub enum PostingPrice<D> {
    /// Unit cost (`@`)
    Unit(Amount<D>),
    /// Total cost (`@@`)
    Total(Amount<D>),
}

/// Transaction tag
///
/// # Example
/// ```
/// # use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = r#"
/// 2022-05-22 * "Grocery store" "Grocery shopping" #food
///   Assets:Cash           -10 CHF
///   Expenses:Groceries
/// "#;
///
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Transaction(trx) = &beancount.directives[0].content else {
///   unreachable!("was not a transaction")
/// };
/// assert!(trx.tags.contains("food"));
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Tag(Arc<str>);

impl Tag {
    /// Returns underlying string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for Tag {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

/// Transaction link
///
/// # Example
/// ```
/// # use beancount_parser::{BeancountFile, DirectiveContent};
/// let input = r#"
/// 2014-02-05 * "Invoice for January" ^invoice-pepe-studios-jan14
///    Income:Clients:PepeStudios           -8450.00 USD
///    Assets:AccountsReceivable
/// "#;
///
/// let beancount: BeancountFile<f64> = input.parse().unwrap();
/// let DirectiveContent::Transaction(trx) = &beancount.directives[0].content else {
///   unreachable!("was not a transaction")
/// };
/// assert!(trx.links.contains("invoice-pepe-studios-jan14"));
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Link(Arc<str>);

impl Link {
    /// Returns underlying string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for Link {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Borrow<str> for Link {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn parse<D: Decimal>(
    input: Span<'_>,
) -> IResult<'_, (Transaction<D>, metadata::Map<D>)> {
    let (input, flag) = alt((map(flag, Some), value(None, tag("txn"))))(input)?;
    cut(do_parse(flag))(input)
}

fn flag(input: Span<'_>) -> IResult<'_, char> {
    satisfy(|c: char| !c.is_ascii_lowercase())(input)
}

fn do_parse<D: Decimal>(
    flag: Option<char>,
) -> impl Fn(Span<'_>) -> IResult<'_, (Transaction<D>, metadata::Map<D>)> {
    move |input| {
        let (input, payee_and_narration) = opt(preceded(space1, payee_and_narration))(input)?;
        let (input, (tags, links)) = tags_and_links(input)?;
        let (input, ()) = end_of_line(input)?;
        let (input, metadata) = metadata::parse(input)?;
        let mut iter = iterator(input, alt((posting.map(Some), empty_line.map(|()| None))));
        let postings = iter.flatten().collect();
        let (input, ()) = iter.finish()?;
        let (payee, narration) = match payee_and_narration {
            Some((payee, narration)) => (payee, Some(narration)),
            None => (None, None),
        };
        Ok((
            input,
            (
                Transaction {
                    flag,
                    payee,
                    narration,
                    tags,
                    links,
                    postings,
                },
                metadata,
            ),
        ))
    }
}

pub(super) enum TagOrLink {
    Tag(Tag),
    Link(Link),
}

pub(super) fn parse_tag(input: Span<'_>) -> IResult<'_, Tag> {
    map(
        preceded(
            char_tag('#'),
            take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
        ),
        |s: Span<'_>| Tag((*s.fragment()).into()),
    )(input)
}

pub(super) fn parse_link(input: Span<'_>) -> IResult<'_, Link> {
    map(
        preceded(
            char_tag('^'),
            take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == '.'),
        ),
        |s: Span<'_>| Link((*s.fragment()).into()),
    )(input)
}

pub(super) fn parse_tag_or_link(input: Span<'_>) -> IResult<'_, TagOrLink> {
    alt((
        map(parse_tag, TagOrLink::Tag),
        map(parse_link, TagOrLink::Link),
    ))(input)
}

fn tags_and_links(input: Span<'_>) -> IResult<'_, (HashSet<Tag>, HashSet<Link>)> {
    let mut tags_and_links_iter = iterator(input, preceded(space0, parse_tag_or_link));
    let (tags, links) = tags_and_links_iter.fold(
        (HashSet::new(), HashSet::new()),
        |(mut tags, mut links), x| {
            match x {
                TagOrLink::Tag(tag) => tags.insert(tag),
                TagOrLink::Link(link) => links.insert(link),
            };
            (tags, links)
        },
    );
    let (input, ()) = tags_and_links_iter.finish()?;
    Ok((input, (tags, links)))
}

fn payee_and_narration(input: Span<'_>) -> IResult<'_, (Option<String>, String)> {
    let (input, s1) = string(input)?;
    let (input, s2) = opt(preceded(space1, string))(input)?;
    Ok((
        input,
        match s2 {
            Some(narration) => (Some(s1), narration),
            None => (None, s1),
        },
    ))
}

fn posting<D: Decimal>(input: Span<'_>) -> IResult<'_, Posting<D>> {
    let (input, _) = space1(input)?;
    let (input, flag) = opt(terminated(flag, space1))(input)?;
    let (input, account) = account::parse(input)?;
    let (input, amounts) = opt(tuple((
        preceded(space1, amount::parse),
        opt(preceded(space1, cost)),
        opt(preceded(
            space1,
            alt((
                map(
                    preceded(tuple((char_tag('@'), space1)), amount::parse),
                    PostingPrice::Unit,
                ),
                map(
                    preceded(tuple((tag("@@"), space1)), amount::parse),
                    PostingPrice::Total,
                ),
            )),
        )),
    )))(input)?;
    let (input, ()) = end_of_line(input)?;
    let (input, metadata) = metadata::parse(input)?;
    let (amount, cost, price) = match amounts {
        Some((a, l, p)) => (Some(a), l, p),
        None => (None, None, None),
    };
    Ok((
        input,
        Posting {
            flag,
            account,
            amount,
            cost,
            price,
            metadata,
        },
    ))
}

fn cost<D: Decimal>(input: Span<'_>) -> IResult<'_, Cost<D>> {
    let (input, _) = terminated(char_tag('{'), space0)(input)?;
    let (input, (cost, date)) = alt((
        map(
            separated_pair(
                amount::parse,
                delimited(space0, char_tag(','), space0),
                date::parse,
            ),
            |(a, d)| (Some(a), Some(d)),
        ),
        map(
            separated_pair(
                date::parse,
                delimited(space0, char_tag(','), space0),
                amount::parse,
            ),
            |(d, a)| (Some(a), Some(d)),
        ),
        map(amount::parse, |a| (Some(a), None)),
        map(date::parse, |d| (None, Some(d))),
        map(success(true), |_| (None, None)),
    ))(input)?;
    let (input, _) = preceded(space0, char_tag('}'))(input)?;
    Ok((input, Cost { amount: cost, date }))
}

#[cfg(test)]
mod chumsky {
    use std::collections::HashSet;

    use crate::{ChumskyParser, Decimal, Posting, PostingPrice, Transaction};
    use chumsky::{prelude::*, text::whitespace};

    use super::{Cost, Link, Tag};

    fn transaction<D: Decimal + 'static>() -> impl ChumskyParser<Transaction<D>> {
        flag()
            .then(payee_and_narration())
            .then(tags_and_links())
            .map(|((flag, (payee, narration)), (tags, links))| Transaction {
                flag,
                payee,
                narration,
                tags,
                links,
                postings: Vec::new(),
            })
    }

    fn flag() -> impl ChumskyParser<Option<char>> {
        choice((
            just("txn").to(None),
            just('!').map(Some),
            just('*').map(Some),
        ))
    }

    fn payee_and_narration() -> impl ChumskyParser<(Option<String>, Option<String>)> {
        whitespace()
            .ignore_then(crate::chumksy::string())
            .then(whitespace().ignore_then(crate::chumksy::string()).or_not())
            .or_not()
            .map(|v| match v {
                Some((p, Some(n))) => (Some(p), Some(n)),
                Some((n, None)) => (None, Some(n)),
                None => (None, None),
            })
    }

    fn tags_and_links() -> impl ChumskyParser<(HashSet<Tag>, HashSet<Link>)> {
        whitespace()
            .ignore_then(just('#'))
            .ignore_then(
                filter(|c: &char| c.is_alphanumeric())
                    .or(one_of("_-"))
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .map(|s| super::Tag(s.into())),
            )
            .padded()
            .repeated()
            .collect()
            .map(|t| (t, HashSet::new()))
    }

    fn posting<D: Decimal + 'static>() -> impl ChumskyParser<Posting<D>> {
        one_of("*!")
            .then_ignore(whitespace())
            .or_not()
            .then(crate::account::chumksy::account())
            .then_ignore(whitespace())
            .then(crate::amount::chumsky::amount().or_not())
            .then(whitespace().ignore_then(cost::<D>()).or_not())
            .then(
                choice((
                    just('@')
                        .padded()
                        .ignore_then(crate::amount::chumsky::amount())
                        .map(PostingPrice::Unit),
                    just("@@")
                        .padded()
                        .ignore_then(crate::amount::chumsky::amount())
                        .map(PostingPrice::Total),
                ))
                .or_not(),
            )
            .then(
                crate::metadata::chumsky::map()
                    .padded()
                    .or_not()
                    .map(Option::unwrap_or_default),
            )
            .map(
                |(((((flag, account), amount), cost), price), metadata)| Posting {
                    flag,
                    account,
                    amount,
                    cost,
                    price,
                    metadata,
                },
            )
    }

    fn cost<D: Decimal + 'static>() -> impl ChumskyParser<Cost<D>> {
        choice((
            crate::amount::chumsky::amount()
                .then(
                    just(',')
                        .padded()
                        .ignore_then(crate::date::chumsky::date())
                        .or_not(),
                )
                .map(|(amount, date)| Cost {
                    amount: Some(amount),
                    date,
                }),
            crate::date::chumsky::date()
                .then(
                    just(',')
                        .padded()
                        .ignore_then(crate::amount::chumsky::amount())
                        .or_not(),
                )
                .map(|(date, amount)| Cost {
                    amount,
                    date: Some(date),
                }),
        ))
        .or_not()
        .padded()
        .delimited_by(just('{'), just('}'))
        .map(Option::unwrap_or_default)
        .labelled("cost")
    }

    #[cfg(test)]
    mod tests {
        use crate::{metadata, transaction::Tag, Amount, Date, PostingPrice, Transaction};

        use super::*;
        use rstest::rstest;

        #[rstest]
        #[case("txn", None)]
        #[case("*", Some('*'))]
        #[case("!", Some('!'))]
        fn should_parse_transaction_flag(#[case] input: &str, #[case] expected: Option<char>) {
            let trx: Transaction<i32> = transaction().parse(input).unwrap();
            assert_eq!(trx.flag, expected);
        }

        #[rstest]
        #[case("*", None, None)]
        #[case("* \"Hello\"", None, Some("Hello"))]
        #[case("* \"Hello\" \"World\"", Some("Hello"), Some("World"))]
        fn should_parse_transaction_description_and_payee(
            #[case] input: &str,
            #[case] expected_payee: Option<&str>,
            #[case] expected_narration: Option<&str>,
        ) {
            let trx: Transaction<i32> = transaction().parse(input).unwrap();
            assert_eq!(trx.payee.as_deref(), expected_payee);
            assert_eq!(trx.narration.as_deref(), expected_narration);
        }

        #[rstest]
        #[case("* \"hello\" \"world\"", &[])]
        #[case("* \"hello\" \"world\" #foo #hello-world", &["foo", "hello-world"])]
        #[case("* \"hello\" \"world\" #2023_05", &["2023_05"])]
        fn should_parse_transaction_tags(#[case] input: &str, #[case] expected: &[&str]) {
            let expected: HashSet<Tag> = expected.iter().map(|s| Tag((*s).into())).collect();
            let trx = transaction::<i32>()
                .then_ignore(end())
                .parse(input)
                .unwrap();
            assert_eq!(trx.tags, expected);
        }

        #[rstest]
        #[case::invalid_tag("* #")]
        #[case::invalid_tag("* #!")]
        fn should_not_parse_invalid_transaction(#[case] input: &str) {
            let result: Result<Transaction<i32>, _> = transaction().then_ignore(end()).parse(input);
            assert!(result.is_err(), "{result:?}");
        }

        #[rstest]
        fn should_parse_posting_account() {
            let posting: Posting<i32> = posting().parse("Assets:Cash").unwrap();
            assert_eq!(posting.account.as_str(), "Assets:Cash");
        }

        #[rstest]
        #[case::none("Assets:Cash", None)]
        #[case::some("Assets:Cash 42 PLN", Some(Amount { value: 42, currency: "PLN".parse().unwrap() }))]
        fn should_parse_posting_amount(#[case] input: &str, #[case] expected: Option<Amount<i32>>) {
            let posting: Posting<i32> = posting().parse(input).unwrap();
            assert_eq!(posting.amount, expected);
        }

        #[rstest]
        #[case::no_flag("Assets:Cash 1 CHF", None)]
        #[case::cleared("* Assets:Cash 1 CHF", Some('*'))]
        #[case::pending("! Assets:Cash 1 CHF", Some('!'))]
        fn should_parse_posting_flag(#[case] input: &str, #[case] expected: Option<char>) {
            let posting: Posting<i32> = posting().parse(input).unwrap();
            assert_eq!(posting.flag, expected);
        }

        #[rstest]
        #[case::none("Assets:Cash 1 CHF", None)]
        #[case::unit("Assets:Cash 1 CHF @ 2 EUR", Some(PostingPrice::Unit(Amount { value: 2, currency: "EUR".parse().unwrap() })))]
        #[case::total("Assets:Cash 1 CHF @@ 2 EUR", Some(PostingPrice::Total(Amount { value: 2, currency: "EUR".parse().unwrap() })))]
        fn should_parse_posting_price(
            #[case] input: &str,
            #[case] expected: Option<PostingPrice<i32>>,
        ) {
            let posting: Posting<i32> = posting().parse(input).unwrap();
            assert_eq!(posting.price, expected);
        }

        #[rstest]
        #[case::none("Assets:Cash 1 CHF", None)]
        #[case::empty("Assets:Cash 1 CHF {}", Some(Cost::default()))]
        #[case::some("Assets:Cash 1 CHF {2023-03-03}", Some(Cost { date: Some(Date::new(2023,3,3)), ..Cost::default() }))]
        #[case::some_before_price("Assets:Cash 1 CHF {2023-03-03} @ 3 PLN", Some(Cost { date: Some(Date::new(2023,3,3)), ..Cost::default() }))]
        fn should_parse_posting_cost(#[case] input: &str, #[case] expected: Option<Cost<i32>>) {
            let posting: Posting<i32> = posting().parse(input).unwrap();
            assert_eq!(posting.cost, expected);
        }

        #[rstest]
        fn should_parse_posting_metadata() {
            let input = "Assets:Cash 10 CHF @ 40 PLN\n  hello: \"world\"";
            let posting: Posting<i32> = posting().parse(input).unwrap();
            assert_eq!(
                posting.metadata.get("hello"),
                Some(&metadata::Value::String("world".into()))
            );
        }

        #[rstest]
        fn should_parse_empty_cost(#[values("{}", "{ }")] input: &str) {
            let cost: Cost<i32> = cost().parse(input).unwrap();
            assert_eq!(cost.amount, None);
            assert_eq!(cost.date, None);
        }

        #[rstest]
        fn should_parse_cost_amount(
            #[values("{1 EUR}", "{ 1 EUR }, {2024-03-03, 1 EUR}")] input: &str,
        ) {
            let cost: Cost<i32> = cost().parse(input).unwrap();
            let amount = cost.amount.unwrap();
            assert_eq!(amount.value, 1);
            assert_eq!(amount.currency.as_str(), "EUR");
        }

        #[rstest]
        fn should_parse_cost_date(
            #[values("{2024-03-02}", "{ 1 EUR , 2024-03-02 }", "{ 2024-03-02, 2 EUR }")]
            input: &str,
        ) {
            let cost: Cost<i32> = cost().parse(input).unwrap();
            assert_eq!(
                cost.date,
                Some(Date {
                    year: 2024,
                    month: 3,
                    day: 2,
                })
            );
        }

        #[rstest]
        #[case::duplicated_date("{2023-03-03, 2023-03-04}")]
        #[case::duplicated_amount("{1 EUR, 2 CHF}")]
        fn should_not_parse_invalid_cost(#[case] input: &str) {
            let result: Result<Cost<i32>, _> = cost().parse(input);
            assert!(result.is_err(), "{result:?}");
        }
    }
}
