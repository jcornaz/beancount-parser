use std::fmt::{Debug, Display, Formatter};

use crate::Span;

/// Error returned in case of invalid beancount syntax found
///
/// # Example
/// ```
/// # use beancount_parser::BeancountFile;
/// let result: Result<BeancountFile<f64>, beancount_parser::Error> = "2022-05-21 oops".parse();
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

/// Error returned when reading a beancount file from disk
#[derive(Debug)]
#[allow(missing_docs, clippy::module_name_repetitions)]
pub enum ReadFileError {
    Io(std::io::Error),
    Syntax(Error),
}

impl Display for ReadFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadFileError::Io(err) => write!(f, "IO error: {err}"),
            ReadFileError::Syntax(err) => write!(f, "Syntax error: {err}"),
        }
    }
}

impl std::error::Error for ReadFileError {}

impl From<std::io::Error> for ReadFileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<Error> for ReadFileError {
    fn from(value: Error) -> Self {
        Self::Syntax(value)
    }
}
