use std::fmt::{Debug, Display, Formatter};

use crate::Span;

/// Error returned in case of invalid beancount syntax found
///
/// # Example
/// ```
/// let result = beancount_parser::parse::<f64>("2022-05-21 oops");
/// assert!(result.is_err());
/// let error = result.unwrap_err();
/// assert_eq!(error.line_number(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Error {
    line_number: u32,
}

impl Error {
    pub(crate) fn new(span: Span<'_>) -> Self {
        Self {
            line_number: span.location_line(),
        }
    }

    /// Line number at which the error was found in the input
    #[must_use]
    pub fn line_number(&self) -> u32 {
        self.line_number
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid beancount syntax at line: {}",
            self.line_number()
        )
    }
}

impl std::error::Error for Error {}
