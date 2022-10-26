mod utils;

use beancount_parser::Parser;
use rstest::rstest;

const SIMPLE: &str = include_str!("examples/simple.beancount");
const COMMENTS: &str = include_str!("examples/comments.beancount");
const OFFICIAL: &str = include_str!("examples/official.beancount");

#[rstest]
fn valid_examples_should_not_return_an_error(
    #[values("", " \n ", SIMPLE, COMMENTS, OFFICIAL)] input: &str,
) {
    let mut count = 0;
    for result in Parser::new(input) {
        count += 1;
        assert!(
            result.is_ok(),
            "The {count} directive is an error: {result:?}"
        );
    }
}
