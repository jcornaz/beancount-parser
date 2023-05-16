use beancount_parser::Flag;
use rstest::rstest;

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
