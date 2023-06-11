use std::fmt::{Debug, Display, Formatter};

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
pub struct Error<'a>(Span<'a>);

impl<'a> Debug for Error<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("line_number", &self.line_number())
            .finish()
    }
}

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
