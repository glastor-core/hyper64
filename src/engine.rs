//! Drop-in compatibility layer with `crates.io/crates/base64` (v0.22).
//!
//! Enables friction-free migration: applications can switch to `hyper64` in `Cargo.toml`
//! without altering existing code or imports.
//!
//! # Example
//! ```rust
//! use hyper64::engine::Engine;
//! use hyper64::engine::general_purpose::STANDARD;
//!
//! let encoded = STANDARD.encode(b"Drop-in Base64 Compatibility");
//! let decoded = STANDARD.decode(&encoded).unwrap();
//! assert_eq!(decoded, b"Drop-in Base64 Compatibility");
//! ```

use crate::alphabet::Config;
use crate::error::DecodeError;
use crate::{decode_to_slice, encode_to_slice};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Core engine trait mirroring `base64::Engine`.
pub trait Engine: Send + Sync {
    /// Encodes input bytes into a newly allocated [`String`].
    #[cfg(feature = "alloc")]
    fn encode<T: AsRef<[u8]>>(&self, input: T) -> String;

    /// Encodes input bytes into a pre-allocated [`Vec<u8>`].
    #[cfg(feature = "alloc")]
    fn encode_vec<T: AsRef<[u8]>>(&self, input: T, output: &mut Vec<u8>);

    /// Encodes input bytes into a caller-provided destination slice.
    #[allow(clippy::result_unit_err)]
    fn encode_slice<T: AsRef<[u8]>>(&self, input: T, output: &mut [u8]) -> Result<usize, ()>;

    /// Decodes Base64 data into a newly allocated [`Vec<u8>`].
    #[cfg(feature = "alloc")]
    fn decode<T: AsRef<[u8]>>(&self, input: T) -> Result<Vec<u8>, DecodeError>;

    /// Decodes Base64 data into a pre-allocated [`Vec<u8>`].
    #[cfg(feature = "alloc")]
    fn decode_vec<T: AsRef<[u8]>>(&self, input: T, output: &mut Vec<u8>)
    -> Result<(), DecodeError>;

    /// Decodes Base64 data into a caller-provided destination slice.
    fn decode_slice<T: AsRef<[u8]>>(
        &self,
        input: T,
        output: &mut [u8],
    ) -> Result<usize, DecodeError>;
}

/// A general-purpose Base64 engine wrapping a [`Config`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneralPurpose {
    config: Config,
}

impl GeneralPurpose {
    /// Creates a new `GeneralPurpose` engine from a given [`Config`].
    pub const fn new(config: &'static Config) -> Self {
        Self { config: *config }
    }

    /// Returns the internal configuration of this engine.
    pub const fn config(&self) -> &Config {
        &self.config
    }
}

impl Engine for GeneralPurpose {
    #[cfg(feature = "alloc")]
    #[inline]
    fn encode<T: AsRef<[u8]>>(&self, input: T) -> String {
        crate::encode::encode(input, &self.config)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn encode_vec<T: AsRef<[u8]>>(&self, input: T, output: &mut Vec<u8>) {
        let in_bytes = input.as_ref();
        let needed = crate::scalar::encoded_len(in_bytes.len(), self.config.padding);
        let current_len = output.len();
        output.resize(current_len + needed, 0);
        encode_to_slice(in_bytes, &mut output[current_len..], &self.config);
    }

    #[allow(clippy::result_unit_err)]
    #[inline]
    fn encode_slice<T: AsRef<[u8]>>(&self, input: T, output: &mut [u8]) -> Result<usize, ()> {
        let in_bytes = input.as_ref();
        let needed = crate::scalar::encoded_len(in_bytes.len(), self.config.padding);
        if output.len() < needed {
            return Err(());
        }
        Ok(encode_to_slice(in_bytes, output, &self.config))
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode<T: AsRef<[u8]>>(&self, input: T) -> Result<Vec<u8>, DecodeError> {
        crate::decode::decode(input, &self.config)
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn decode_vec<T: AsRef<[u8]>>(
        &self,
        input: T,
        output: &mut Vec<u8>,
    ) -> Result<(), DecodeError> {
        let bytes = input.as_ref();
        let min_cap = (bytes.len() / 4) * 3;
        let current_len = output.len();
        output.resize(current_len + min_cap, 0);
        let written = decode_to_slice(bytes, &mut output[current_len..], &self.config)?;
        output.truncate(current_len + written);
        Ok(())
    }

    #[inline]
    fn decode_slice<T: AsRef<[u8]>>(
        &self,
        input: T,
        output: &mut [u8],
    ) -> Result<usize, DecodeError> {
        decode_to_slice(input.as_ref(), output, &self.config)
    }
}

/// Pre-configured standard and URL-safe general-purpose engines.
pub mod general_purpose {
    use super::GeneralPurpose;
    use crate::alphabet::Config;

    /// Standard Base64 engine with '=' padding.
    pub const STANDARD: GeneralPurpose = GeneralPurpose::new(&Config::STANDARD);
    /// Standard Base64 engine without padding.
    pub const STANDARD_NO_PAD: GeneralPurpose = GeneralPurpose::new(&Config::STANDARD_NO_PAD);
    /// URL-safe and filename-safe Base64 engine with '=' padding.
    pub const URL_SAFE: GeneralPurpose = GeneralPurpose::new(&Config::URL_SAFE);
    /// URL-safe and filename-safe Base64 engine without padding.
    pub const URL_SAFE_NO_PAD: GeneralPurpose = GeneralPurpose::new(&Config::URL_SAFE_NO_PAD);
}

/// Common prelude re-exporting engines and the [`Engine`] trait.
pub mod prelude {
    pub use super::Engine;
    pub use super::general_purpose::{
        STANDARD as BASE64_STANDARD, STANDARD_NO_PAD as BASE64_STANDARD_NO_PAD,
        URL_SAFE as BASE64_URL_SAFE, URL_SAFE_NO_PAD as BASE64_URL_SAFE_NO_PAD,
    };
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::prelude::*;
    use alloc::vec::Vec;

    #[test]
    fn test_engine_trait_roundtrip() {
        let data = b"hyper64 drop-in engine test";
        let encoded = BASE64_STANDARD.encode(data);
        let decoded = BASE64_STANDARD.decode(&encoded).unwrap();
        assert_eq!(&decoded, data);

        let mut enc_vec = Vec::new();
        BASE64_STANDARD.encode_vec(data, &mut enc_vec);
        assert_eq!(enc_vec, encoded.as_bytes());

        let mut dec_vec = Vec::new();
        BASE64_STANDARD.decode_vec(&encoded, &mut dec_vec).unwrap();
        assert_eq!(&dec_vec, data);
    }
}
