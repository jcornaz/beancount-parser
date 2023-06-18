use std::fmt::Debug;

use rstest::rstest;

use beancount_parser_2::{parse, BeancountFile, DirectiveContent, Flag};

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
fn default_flag_is_completed() {
    assert_eq!(Flag::default(), Flag::Completed);
}

#[rstest]
#[case(Flag::Completed, '*')]
#[case(Flag::Incomplete, '!')]
fn can_convert_from_flag_to_char(#[case] flag: Flag, #[case] expected: char) {
    let actual: char = flag.into();
    assert_eq!(actual, expected);
}

#[test]
fn error_debug_impl_is_succinct() {
    let input = "2023-06-11 * Oops\n\n\n\n\n; end comment";
    let Err(err) = parse::<f64>(input) else { unreachable!("parsing should fail") };
    let debug = format!("{err:?}");
    assert!(!debug.contains("; end comment"), "{}", debug);
}

#[rstest]
fn accounts_implements_display() {
    let account = "Expenses:Taxes:Y2021:US:Federal:PreTax401k";
    let input = format!("2023-06-18 open {account}");
    let DirectiveContent::Open(ref open) = parse::<f64>(&input).unwrap().directives[0].content else {
        unreachable!("was not an open directive")
    };
    let actual = format!("{}", open.account);
    assert_eq!(&actual, account);
}

#[rstest]
fn currency_implements_display() {
    let input = "2023-06-18 commodity CHF";
    let DirectiveContent::Commodity(ref currency) = parse::<f64>(input).unwrap().directives[0].content else {
        unreachable!("was not an open directive")
    };
    assert_eq!(&format!("{currency}"), "CHF");
}
