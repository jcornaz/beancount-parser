use beancount_parser::{Directive, Parser};

pub trait DirectiveList<'a> {
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
