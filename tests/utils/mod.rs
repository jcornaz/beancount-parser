#![allow(unused)] // not all tests use everything

use beancount_parser::{Date, Directive, Parser};

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

pub fn assert_date_eq(date: Date, year: u16, month_of_year: u8, day_of_month: u8) {
    assert_eq!(
        date.year(),
        year,
        "unexpected year in {date:?} (expecting year {year})"
    );
    assert_eq!(
        date.month_of_year(),
        month_of_year,
        "unexpected month in {date:?} (expecting month {month_of_year})"
    );
    assert_eq!(
        date.day_of_month(),
        day_of_month,
        "unexpected day in {date:?} (expecting day {day_of_month})"
    );
}
