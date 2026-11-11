//! Zero-allocation Base64 formatting for `core::fmt`.
//!
//! Avoids allocating intermediate heap strings when logging, serializing,
//! writing to HTTP streams, or generating headers.

use crate::alphabet::Config;
use crate::scalar::encode_scalar;
use core::fmt;

/// Wrapper around a byte slice that implements [`core::fmt::Display`] to emit
/// Base64 ASCII directly to a formatter without heap allocations.
#[derive(Debug, Clone, Copy)]
pub struct Base64Display<'a> {
    data: &'a [u8],
    config: &'static Config,
}

impl<'a> Base64Display<'a> {
    /// Creates a new display wrapper with the given configuration.
    #[inline(always)]
    pub const fn new(data: &'a [u8], config: &'static Config) -> Self {
        Self { data, config }
    }

    /// Creates a new display wrapper with the standard Base64 configuration.
    #[inline(always)]
    pub const fn standard(data: &'a [u8]) -> Self {
        Self {
            data,
            config: &Config::STANDARD,
        }
    }

    /// Creates a new display wrapper with the URL-safe unpadded configuration.
    #[inline(always)]
    pub const fn url_safe(data: &'a [u8]) -> Self {
        Self {
            data,
            config: &Config::URL_SAFE_NO_PAD,
        }
    }
}

impl<'a> fmt::Display for Base64Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Process in 48-byte chunks -> 64 ASCII bytes per chunk
        // Fits entirely in a single L1 cache line without heap allocation
        let mut buf = [0u8; 64];
        let chunks = self.data.chunks(48);

        for chunk in chunks {
            let written = encode_scalar(chunk, &mut buf, self.config);
            let s = unsafe { core::str::from_utf8_unchecked(&buf[..written]) };
            f.write_str(s)?;
        }

        Ok(())
    }
}

/// Formats `data` as standard Base64 into any formatter without heap allocations.
#[inline(always)]
pub fn display<'a>(data: &'a [u8]) -> Base64Display<'a> {
    Base64Display::standard(data)
}

/// Formats `data` using the specified configuration without heap allocations.
#[inline(always)]
pub fn display_config<'a>(data: &'a [u8], config: &'static Config) -> Base64Display<'a> {
    Base64Display::new(data, config)
}

/// Formats `data` as URL-safe unpadded Base64 without heap allocations.
#[inline(always)]
pub fn display_url_safe<'a>(data: &'a [u8]) -> Base64Display<'a> {
    Base64Display::url_safe(data)
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn test_display_formatting() {
        extern crate alloc;
        use alloc::format;

        let data = b"Hello Rust 2026";
        let formatted = format!("{}", display(data));
        assert_eq!(formatted, "SGVsbG8gUnVzdCAyMDI2");

        let formatted_url = format!("{}", display_url_safe(data));
        assert_eq!(formatted_url, "SGVsbG8gUnVzdCAyMDI2");
    }
}
