#![cfg(all(test, feature = "unstable"))]

use pest::Parser as Parse;
use pest_derive::Parser;

use crate::{account, Close, Date, Directive, Open, Transaction};

#[derive(Parser)]
#[grammar = "beancount.pest"]
struct Parser;

pub(crate) type Pair<'a> = pest::iterators::Pair<'a, Rule>;

fn parse(input: &str) -> Result<impl Iterator<Item = Directive<'_>>, Box<dyn std::error::Error>> {
    Ok(Parser::parse(Rule::beancount_file, input)?
        .next()
        .expect("no root rule")
        .into_inner()
        .filter_map(|p| match p.as_rule() {
            Rule::transaction => Some(Directive::Transaction(Transaction::from_pair(p))),
            Rule::open => Some(Directive::Open(Open::from_pair(p))),
            Rule::close => Some(Directive::Close(Close::from_pair(p))),
            _ => None,
        }))
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::amount::Expression;
    use crate::transaction::{Flag, Posting};
    use crate::Amount;

    use super::*;

    const COMMENTS: &str = include_str!("../tests/samples/comments.beancount");
    const SIMPLE: &str = include_str!("../tests/samples/simple.beancount");
    // TODO const OFFICIAL: &str = include_str!("../tests/samples/official.beancount");

    #[rstest]
    fn successful_parse(#[values("", " ", " \n ", " \t ", COMMENTS, SIMPLE)] input: &str) {
        if let Err(err) = parse(input) {
            panic!("{err}");
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
        let count = parse(input)
            .expect("should successfully parse input")
            .count();
        assert_eq!(count, 0);
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
    #[case("2022-02-12 *", None, None)]
    #[case("2022-02-12 txn \"Hello\"", None, Some("Hello"))]
    #[case("2022-02-12 * \"Hello\"", None, Some("Hello"))]
    #[case("2022-02-12 txn \"Hello\" \"World\"", Some("Hello"), Some("World"))]
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
    #[case("2022-02-12 txn", &[])]
    #[case("2022-02-12 txn\n  Assets:Hello\n\tExpenses:Test \nLiabilities:Other", &["Assets:Hello", "Expenses:Test", "Liabilities:Other"])]
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
    #[case("! Assets:Hello", Some(Flag::Pending))]
    #[case("  ! Assets:Hello", Some(Flag::Pending))]
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
    #[case("Assets:Hello 10 CHF ;;;  World", Some("World"))]
    #[case("Assets:Hello 10 CHF; Tadaa", Some("Tadaa"))]
    fn parse_posting_comment(#[case] input: &str, #[case] expected: Option<&str>) {
        let input = format!("2022-02-23 txn\n{input}");
        let transaction = parse_single_directive(&input).into_transaction().unwrap();
        let posting = &transaction.postings()[0];
        assert_eq!(posting.comment(), expected);
    }

    #[rstest]
    #[case("2022-02-12 txn\n  Assets:Hello", None)]
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
    #[case("2 / 3", Expression::div(Expression::value(2), Expression::value(3)))]
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
    #[case("2016-11-28 open Assets CHF,EUR", &["CHF", "EUR"])]
    #[case("2016-11-28 open Assets CHF , EUR", &["CHF", "EUR"])]
    #[case("2016-11-28 open Assets AB-CD, A_2B, A.B, A'B", &["AB-CD", "A_2B", "A.B", "A'B"])]
    fn parse_open_directive_currencies(#[case] input: &str, #[case] expected: &[&str]) {
        let directive = parse_single_directive(input);
        let Directive::Open(open) = directive else { panic!("expected open directive but was {directive:?}") };
        assert_eq!(open.currencies(), expected);
    }

    #[rstest]
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
            "2022-02-12 txn\n  Assets:Hello  10CHF",
            "2022-02-12 txn\n  Assets:Hello10 CHF",
            "2022-02-12 txn\n  Assets:Hello 1 +  CHF",
            "2022-02-12 txn\n  Assets:Hello 2 *  CHF",
            "2022-02-12 txn\n  Assets:Hello 1 /  CHF"
        )]
        input: &str,
    ) {
        let result = parse(input);
        assert!(result.is_err());
    }

    fn parse_single_directive(input: &str) -> Directive<'_> {
        let mut iter = match parse(input) {
            Ok(iter) => iter,
            Err(err) => panic!("{err}"),
        };
        let directive = iter.next().expect("There was no directives");
        assert!(iter.next().is_none(), "There was more than one directive");
        directive
    }
}
