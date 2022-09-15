#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Error;

use crate::ConversionError;

impl From<ConversionError> for Error {
    fn from(_: ConversionError) -> Self {
        Self
    }
}
