use std::backtrace::Backtrace;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Conversion error: {expected} (expected) != {actual} (actual)")]
pub struct ConversionError {
    pub expected: String,
    pub actual: String,
    pub backtrace: Backtrace,
}
