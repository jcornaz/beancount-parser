mod utils;

use beancount_parser::{Directive, Parser};

use crate::utils::{assert_date_eq, DirectiveList};

#[test]
fn close_directive() {
    let input = "2016-11-28 close Liabilities:CreditCard:CapitalOne";
    let directive = match Parser::new(input).assert_single_directive() {
        Directive::Close(d) => d,
        d => panic!("unexpected directive type: {d:?}"),
    };
    assert_date_eq(directive.date(), 2016, 11, 28);
    assert_eq!(
        directive.account().components(),
        &["CreditCard", "CapitalOne"]
    );
}
