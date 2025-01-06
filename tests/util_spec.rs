#![allow(missing_docs)]

use std::fmt::Debug;

use rstest::rstest;

use beancount_parser::{
    parse, parse_iter, BeancountFile, Currency, Date, Directive, DirectiveContent, Entry, Error,
};

fn is_normal<T: Sized + Send + Sync + Unpin>() {}
fn is_debug<T: Debug>() {}
fn is_clone<T: Clone>() {}

#[test]
fn beancount_file_type_should_be_normal() {
    is_normal::<BeancountFile<f32>>();
    is_debug::<BeancountFile<f32>>();
    is_clone::<BeancountFile<f32>>();
}

#[test]
fn result_entry_type_should_be_normal() {
    is_normal::<Result<Entry<f32>, Error>>();
    is_debug::<Result<Entry<f32>, Error>>();
    is_clone::<Result<Entry<f32>, Error>>();
}

#[test]
fn error_debug_impl_is_succinct() {
    let input = "2023-06-11 * Oops\n\n\n\n\n; end comment";
    let err = parse_iter::<f64>(input).next().unwrap().unwrap_err();
    let debug = format!("{err:?}");
    assert!(!debug.contains("; end comment"), "{}", debug);
}

#[rstest]
fn accounts_implements_display() {
    let account = "Expenses:Taxes:Y2021:US:Federal:PreTax401k";
    let input = format!("2023-06-18 open {account}");
    let DirectiveContent::Open(ref open) = parse::<f64>(&input).unwrap().directives[0].content
    else {
        unreachable!("was not an open directive")
    };
    let actual = format!("{}", open.account);
    assert_eq!(&actual, account);
}

#[rstest]
fn currency_implements_display() {
    let input = "2023-06-18 commodity CHF";
    let DirectiveContent::Commodity(ref currency) =
        parse::<f64>(input).unwrap().directives[0].content
    else {
        unreachable!("was not an open directive")
    };
    assert_eq!(&format!("{currency}"), "CHF");
}

#[rstest]
#[case(Date { year: 2023, month: 6, day: 18 }, Date { year: 2024, month: 5, day: 17 })]
#[case(Date { year: 2023, month: 6, day: 18 }, Date { year: 2023, month: 7, day: 17 })]
#[case(Date { year: 2023, month: 6, day: 18 }, Date { year: 2023, month: 7, day: 19 })]
fn date_comparison(#[case] smaller: Date, #[case] bigger: Date) {
    assert!(smaller < bigger);
    assert!(bigger > smaller);
}

#[rstest]
fn can_parse_valid_currency_from_str(#[values("A", "CHF", "USD'42-CHF_EUR.PLN")] raw: &str) {
    let currency: Currency = raw.try_into().unwrap();
    assert_eq!(currency.as_str(), raw);
}

#[rstest]
fn reject_invalid_currency_from_str(
    #[values("hello world", "oops", "1SD", "-US", "US-", "CHF ")] raw: &str,
) {
    let currency: Result<Currency, _> = raw.try_into();
    assert!(currency.is_err());
}

#[rstest]
fn directive_from_str(
    #[values(
        "2023-07-09 close Assets:Cash",
        "2023-07-09 open Assets:Cash CHF",
        "2023-07-09 * \"hello\"",
        "2023-07-09 * \"hello\" \"world\"",
        "2023-07-09 * \"hello\" #hello",
        "2023-07-09 * \"hello\" ^hello",
        "2023-07-09 * \"hello\"\n  title: \"cool\"",
        "2023-07-09 * \"hello\"\n  Assets:Cash 10 CHF\n  Income:Gifts"
    )]
    input: &str,
) {
    let file: BeancountFile<f64> = input.parse().unwrap();
    let from_file = file.directives.into_iter().next().unwrap();
    let from_str: Directive<f64> = input.parse().unwrap();
    assert_eq!(from_file, from_str);
}
