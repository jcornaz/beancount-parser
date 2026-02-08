#![allow(clippy::module_name_repetitions)]
#![allow(deprecated)]
#![allow(unused_assignments)]

use std::{
    fmt::{Debug, Display},
    io,
    path::PathBuf,
};

#[cfg(feature = "miette")]
use miette::{Diagnostic, SourceSpan};

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
#[derive(Clone)]
#[cfg_attr(feature = "miette", derive(Diagnostic))]
pub struct Error {
    #[cfg(feature = "miette")]
    #[source_code]
    src: String,
    #[cfg(feature = "miette")]
    #[label]
    span: SourceSpan,
    line_number: u32,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("line_number", &self.line_number())
            .finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid beancount syntax at line: {}", self.line_number)
    }
}

impl std::error::Error for Error {}

impl Error {
    #[cfg(not(feature = "miette"))]
    pub(crate) fn new(_: impl Into<String>, span: Span<'_>) -> Self {
        Self {
            line_number: span.location_line(),
        }
    }

    #[cfg(feature = "miette")]
    pub(crate) fn new(src: impl Into<String>, span: Span<'_>) -> Self {
        Self {
            src: src.into(),
            span: span.location_offset().into(),
            line_number: span.location_line(),
        }
    }

    /// Line number at which the error was found in the input
    #[must_use]
    pub fn line_number(&self) -> u32 {
        self.line_number
    }
}

/// Error returned when reading a beancount file from disk
#[allow(missing_docs)]
#[derive(Debug)]
pub struct ReadFileErrorV2 {
    path: PathBuf,
    pub(crate) error: ReadFileErrorContent,
}

impl ReadFileErrorV2 {
    pub(crate) fn from_io(path: PathBuf, err: io::Error) -> Self {
        Self {
            path,
            error: ReadFileErrorContent::Io(err),
        }
    }

    pub(crate) fn from_syntax(path: PathBuf, err: Error) -> Self {
        Self {
            path,
            error: ReadFileErrorContent::Syntax(err),
        }
    }
}

impl Display for ReadFileErrorV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            ReadFileErrorContent::Io(err) => {
                write!(f, "Cannot read {}: {}", self.path.display(), err)
            }
            ReadFileErrorContent::Syntax(err) => {
                write!(f, "Invalid syntax in {}: {}", self.path.display(), err)
            }
        }
    }
}

impl std::error::Error for ReadFileErrorV2 {}

/// Content of the error returned when reading a beancount file from disk
#[allow(missing_docs)]
#[derive(Debug)]
pub(crate) enum ReadFileErrorContent {
    Io(std::io::Error),
    Syntax(Error),
}

/// Content of the error returned when reading a beancount file from disk
#[allow(missing_docs)]
#[derive(Debug)]
#[cfg_attr(feature = "miette", derive(Diagnostic))]
#[deprecated(since = "2.4.0", note = "use `ReadFileErrorV2 instead`")]
pub enum ReadFileError {
    Io(std::io::Error),
    Syntax(Error),
}

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

impl Display for ReadFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadFileError::Io(_) => write!(f, "IO error"),
            ReadFileError::Syntax(_) => write!(f, "Syntax error"),
        }
    }
}

impl std::error::Error for ReadFileError {}

/// Error that may be returned by the various `TryFrom`/`TryInto` implementation
/// to signify that the value cannot be converted to the desired type
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ConversionError;

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot convert to the desired type")
    }
}

impl std::error::Error for ConversionError {}
