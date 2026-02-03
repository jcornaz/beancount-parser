#![allow(clippy::module_name_repetitions)]

use std::fmt::Debug;

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
#[derive(Debug, Error)]
#[cfg_attr(feature = "miette", derive(Diagnostic))]
pub enum ReadFileError {
    #[error("{0}")]
    Io(#[source] ReadFileIOError),
    #[error("Syntax error")]
    Syntax(#[from] Error),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub struct ReadFileIOError {
    pub path: std::path::PathBuf,
    pub cause: std::io::Error,
}

impl std::fmt::Display for ReadFileIOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IO error in {}: {}", self.path.display(), self.cause)
    }
}

/// Error that may be returned by the various `TryFrom`/`TryInto` implementation
/// to signify that the value cannot be converted to the desired type
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
#[error("Cannot convert to the desired type")]
pub struct ConversionError;
