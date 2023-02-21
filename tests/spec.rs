use beancount_parser::{Date, Directive, Parser};
use rstest::rstest;

const SIMPLE: &str = include_str!("samples/simple.beancount");
const COMMENTS: &str = include_str!("samples/comments.beancount");
const OFFICIAL: &str = include_str!("samples/official.beancount");

#[rstest]
fn valid_examples_do_not_return_an_error(
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
#[rstest]
#[case("", 0)]
#[case(SIMPLE, 3)]
#[case(COMMENTS, 0)]
fn examples_have_expected_number_of_transaction(
    #[case] input: &str,
    #[case] expected_count: usize,
) {
    let actual_count = Parser::new(input)
        .filter_map(Result::ok)
        .filter_map(Directive::into_transaction)
        .count();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
#[case("", 0)]
#[case(SIMPLE, 13)]
#[case(COMMENTS, 0)]
fn examples_have_expected_number_of_postings(#[case] input: &str, #[case] expected_count: usize) {
    let actual_count: usize = Parser::new(input)
        .filter_map(Result::ok)
        .filter_map(Directive::into_transaction)
        .map(|t| t.postings().len())
        .sum();
    assert_eq!(actual_count, expected_count);
}

#[rstest]
fn invalid_examples_return_an_error(#[values("2022-09-10 txn Oops...")] input: &str) {
    let items = Parser::new(input).collect::<Vec<Result<_, _>>>();
    assert!(items[0].is_err());
}

#[test]
fn pushtags_adds_tag_to_next_transaction() {
    let input = "pushtag #hello\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert_eq!(transaction.tags(), &["hello"]);
}

#[test]
fn multiple_pushtags_add_tags_to_next_transaction() {
    let input = "pushtag #hello\npushtag #world\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert_eq!(transaction.tags(), &["hello", "world"]);
}

#[test]
fn poptag_removes_tag_from_stack() {
    let input = "pushtag #hello\npoptag #hello\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert!(transaction.tags().is_empty());
}

#[test]
fn poptag_removes_only_concerned_tag_from_stack() {
    let input = "pushtag #hello\npushtag #world\npoptag #hello\n2022-10-20 txn";
    let transaction = Parser::new(input)
        .assert_single_directive()
        .into_transaction()
        .expect("should be a transaction");
    assert_eq!(transaction.tags(), &["world"]);
}

#[rstest]
fn comment_line(
    #[values(
        "",
        "\n",
        "2016 - 11 - 28 close Liabilities:CreditCard:CapitalOne",
        "Hello world",
        "* Banking",
        "** Bank of America",
        ";; Transactions follow …",
        "; foo bar"
    )]
    input: &str,
) {
    let directives = Parser::new(input)
        .collect::<Result<Vec<_>, _>>()
        .expect("successful parse");
    assert_eq!(directives.len(), 0);
}

#[test]
fn close_directive() {
    let directive =
        Parser::new("2016-11-28 close Liabilities:CreditCard:CapitalOne").assert_single_directive();
    let Directive::Close(directive) = directive else {
        panic!("Expected a close directive but was: {directive:?}")
    };
    assert_eq!(directive.date(), Date::new(2016, 11, 28));
    assert_eq!(
        directive.account().components(),
        &["CreditCard", "CapitalOne"]
    );
}

trait DirectiveList<'a> {
    fn assert_single_directive(self) -> Directive<'a>;
}

impl<'a> DirectiveList<'a> for Parser<'a> {
    fn assert_single_directive(mut self) -> Directive<'a> {
        let directive = self
            .next()
            .expect("Exactly one element is expected, but none was found")
            .expect("should successfully parse the input");
        let rest = self.count();
        assert_eq!(
            rest,
            0,
            "Exactly one element is expected, but {} than one was found",
            rest + 1
        );
        directive
    }
}
