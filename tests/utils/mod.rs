use beancount_parser::{Directive, Parser};

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
