use beancount_parser::{Directive, Parser, Transaction};

pub fn assert_single_transaction(input: &str) -> Transaction<'_> {
    assert_single_directive(input)
        .into_transaction()
        .expect("was not a transaction")
}

pub fn assert_single_directive(input: &str) -> Directive<'_> {
    let mut parser = Parser::new(input);
    let directive = parser
        .next()
        .expect("Exactly one element is expected, but none was found")
        .expect("should successfully parse the input");
    let rest = parser.count();
    assert_eq!(
        rest,
        0,
        "Exactly one element is expected, but {} than one was found",
        rest + 1
    );
    directive
}
