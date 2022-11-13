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

    pub(crate) fn from_parsing() -> Self {
        Self(Content::ParseError)
    }
}

#[derive(Debug, Clone)]
enum Content {
    ParseError,
    ConversionError(ConversionError),
}
