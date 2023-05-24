use std::fmt::Debug;

use beancount_parser_2::{BeancountFile, Flag};
use rstest::rstest;

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
