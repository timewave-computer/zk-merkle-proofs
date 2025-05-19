//! RLP (Recursive Length Prefix) error handling.
//!
//! This module defines the error types used in RLP encoding and decoding operations.
//! It provides detailed error information for various failure cases that can occur
//! during RLP processing.

use core::fmt;

/// RLP result type.
///
/// This is a type alias for the standard Result type, specialized for RLP operations.
/// It uses the custom `Error` type defined in this module.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// RLP error type.
///
/// This enum represents the various ways in which RLP encoding or decoding can fail.
/// Each variant provides specific information about the nature of the error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Numeric overflow occurred during encoding or decoding.
    Overflow,
    /// A leading zero was found in a number, which is not allowed in RLP.
    LeadingZero,
    /// The input buffer was exhausted before decoding was complete.
    InputTooShort,
    /// A single byte value was expected but an invalid value was found.
    NonCanonicalSingleByte,
    /// An invalid size value was encountered during decoding.
    NonCanonicalSize,
    /// The decoded payload had an unexpected length.
    UnexpectedLength,
    /// A string was found where another type was expected.
    UnexpectedString,
    /// A list was found where another type was expected.
    UnexpectedList,
    /// A list had an unexpected number of items.
    ListLengthMismatch {
        /// The expected number of items in the list.
        expected: usize,
        /// The actual number of items found in the list.
        got: usize,
    },
    /// A custom error message for other failure cases.
    Custom(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("overflow"),
            Self::LeadingZero => f.write_str("leading zero"),
            Self::InputTooShort => f.write_str("input too short"),
            Self::NonCanonicalSingleByte => f.write_str("non-canonical single byte"),
            Self::NonCanonicalSize => f.write_str("non-canonical size"),
            Self::UnexpectedLength => f.write_str("unexpected length"),
            Self::UnexpectedString => f.write_str("unexpected string"),
            Self::UnexpectedList => f.write_str("unexpected list"),
            Self::ListLengthMismatch { got, expected } => {
                write!(f, "unexpected list length (got {got}, expected {expected})")
            }
            Self::Custom(err) => f.write_str(err),
        }
    }
}
