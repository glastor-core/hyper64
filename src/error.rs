//! Error types for hyper64 encoding and decoding operations.

use core::fmt;

/// Errors that can occur during Base64 decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The input length is invalid for the configured padding rules.
    InvalidLength,
    /// An invalid ASCII byte was encountered at the specified position.
    InvalidByte {
        /// The byte offset where the invalid character was found.
        offset: usize,
        /// The invalid byte value.
        byte: u8,
    },
    /// The padding (`=`) is malformed or misplaced.
    InvalidPadding {
        /// The offset where the malformed padding occurred.
        offset: usize,
    },
    /// The destination buffer provided is too small.
    OutputSliceTooSmall {
        /// Expected minimum capacity.
        expected: usize,
        /// Actual capacity provided.
        actual: usize,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidLength => write!(f, "Invalid base64 input length"),
            Self::InvalidByte { offset, byte } => {
                write!(f, "Invalid base64 byte {:#04x} at index {}", byte, offset)
            }
            Self::InvalidPadding { offset } => {
                write!(f, "Invalid padding character at index {}", offset)
            }
            Self::OutputSliceTooSmall { expected, actual } => {
                write!(
                    f,
                    "Destination buffer too small: expected at least {} bytes, got {}",
                    expected, actual
                )
            }
        }
    }
}

// In Rust 2024+, core::error::Error is stabilized in core for #![no_std]
impl core::error::Error for DecodeError {}
