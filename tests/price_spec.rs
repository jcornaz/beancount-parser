mod utils;

use beancount_parser::{Directive, Parser};

use crate::utils::{assert_date_eq, DirectiveList};

#[test]
fn parse_price_directive() {
    let beancount = "2014-07-09 price CHF  5 PLN";
    let directive = match Parser::new(beancount).assert_single_directive() {
        Directive::Price(price) => price,
        d => panic!("Was not a price directive: {d:?}"),
    };
    assert_date_eq(directive.date(), 2014, 7, 9);
    assert_eq!(directive.commodity(), "CHF");
    assert_eq!(directive.price().value().try_into_f64().unwrap(), 5.0);
    assert_eq!(directive.price().currency(), "PLN");
}
