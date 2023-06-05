use std::fmt::Display;

use crate::Span;

/// Error returned in case of invalid beancount syntax found
///
/// # Example
/// ```
/// let result = beancount_parser_2::parse::<f64>("2022-05-21 oops");
/// assert!(result.is_err());
/// let error = result.unwrap_err();
/// assert_eq!(error.line_number(), 1);
/// ```
#[derive(Debug)]
pub struct Error<'a>(Span<'a>);

impl<'a> Error<'a> {
    pub(crate) fn new(span: Span<'a>) -> Self {
        Self(span)
    }

    /// Line number at which the error was found in the input
    #[must_use]
    pub fn line_number(&self) -> u32 {
        self.0.location_line()
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid beancount syntax at line: {}",
            self.line_number()
        )
    }
}

impl<'a> std::error::Error for Error<'a> {}
