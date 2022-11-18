use crate::amount::ConversionError;

/// Main error type
///
/// It doesn't provide any useful information yet.
///
/// It will probably be improved in the future to make possible
/// user-friendly error messages.
#[derive(Debug, Clone)]
pub struct Error(Content);

impl Error {
    pub(crate) fn from_conversion(err: ConversionError) -> Self {
        Self(Content::ConversionError(err))
    }

    #[cfg(feature = "unstable")]
    pub(crate) fn from_parsing(line: u64) -> Self {
        Self(Content::ParseError { line })
    }

    #[cfg(not(feature = "unstable"))]
    pub(crate) fn from_parsing(_: u64) -> Self {
        Self(Content::ParseError {})
    }

    /// Returns the line number at which the error was found in the source text
    ///
    /// The number is 1-based. (The first line number is 1)
    ///
    /// # Panics
    ///
    /// Panic if the error was not a parsing error (i.e. a conversion error)
    #[cfg(feature = "unstable")]
    #[must_use]
    pub fn line_number(&self) -> u64 {
        match self.0 {
            Content::ParseError { line } => line,
            Content::ConversionError(_) => {
                panic!("Cannot get the line number from a conversion error")
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Content {
    ParseError {
        #[cfg(feature = "unstable")]
        line: u64,
    },
    ConversionError(ConversionError),
}
