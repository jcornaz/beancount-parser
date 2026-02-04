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
use thiserror::Error;

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
#[derive(Clone, Error)]
#[cfg_attr(feature = "miette", derive(Diagnostic))]
#[error("Invalid beancount syntax at line: {line_number}")]
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
#[derive(Debug, Error)]
#[cfg_attr(feature = "miette", derive(Diagnostic))]
#[deprecated(since = "2.4.0", note = "use `ReadFileErrorV2 instead`")]
pub enum ReadFileError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Syntax error")]
    Syntax(#[from] Error),
}

/// Error that may be returned by the various `TryFrom`/`TryInto` implementation
/// to signify that the value cannot be converted to the desired type
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
#[error("Cannot convert to the desired type")]
pub struct ConversionError;
