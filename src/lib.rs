#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod alphabet;
pub mod const_fn;
pub mod constant_time;
pub mod decode;
pub mod display;
pub mod encode;
pub mod engine;
pub mod error;
pub mod permissive;
pub mod scalar;
pub mod simd;

#[cfg(feature = "std")]
pub mod io;

#[cfg(feature = "serde")]
pub mod serde;

// Re-exports of key types
pub use alphabet::{
    Config, STANDARD_DECODE_TABLE, STANDARD_ENCODE_TABLE, URL_SAFE_DECODE_TABLE,
    URL_SAFE_ENCODE_TABLE,
};
pub use constant_time::{ct_eq, decode_slice_ct as decode_constant_time};
pub use decode::{decode_in_place, decode_to_slice};
pub use display::{Base64Display, display, display_config, display_url_safe};
pub use encode::{Base64ChunkIter, encode_iter, encode_to_slice};
pub use engine::Engine;
pub use error::DecodeError;
pub use permissive::{PemBlock, decode_pem_in_place, decode_permissive_in_place};
#[cfg(feature = "alloc")]
pub use permissive::{decode_pem, decode_permissive};
pub use scalar::encoded_len;
pub use simd::{has_avx2, has_avx512, has_neon, has_ssse3, has_wasm_simd128};

#[cfg(feature = "std")]
pub use io::{DecoderReader, EncoderWriter};

#[cfg(feature = "tokio")]
pub use io::{AsyncDecoderReader, AsyncEncoderWriter};

#[cfg(feature = "serde")]
pub use crate::serde::Base64;

// Default profile shortcuts
pub const STANDARD: Config = Config::STANDARD;
pub const STANDARD_NO_PAD: Config = Config::STANDARD_NO_PAD;
pub const URL_SAFE: Config = Config::URL_SAFE;
pub const URL_SAFE_NO_PAD: Config = Config::URL_SAFE_NO_PAD;

/// Encodes input bytes to a standard Base64 [`alloc::string::String`].
#[cfg(feature = "alloc")]
#[inline]
pub fn encode(input: impl AsRef<[u8]>) -> alloc::string::String {
    encode::encode(input, &STANDARD)
}

/// Encodes input bytes using the specified configuration.
#[cfg(feature = "alloc")]
#[inline]
pub fn encode_config(input: impl AsRef<[u8]>, config: &Config) -> alloc::string::String {
    encode::encode(input, config)
}

/// Decodes standard Base64 bytes or string into a [`alloc::vec::Vec<u8>`].
#[cfg(feature = "alloc")]
#[inline]
pub fn decode(input: impl AsRef<[u8]>) -> Result<alloc::vec::Vec<u8>, DecodeError> {
    decode::decode(input, &STANDARD)
}

/// Decodes Base64 data using the specified configuration into a [`alloc::vec::Vec<u8>`].
#[cfg(feature = "alloc")]
#[inline]
pub fn decode_config(
    input: impl AsRef<[u8]>,
    config: &Config,
) -> Result<alloc::vec::Vec<u8>, DecodeError> {
    decode::decode(input, config)
}

/// Constant-time encode for cryptographic/secret data (mitigates cache-timing attacks).
#[inline]
pub fn encode_constant_time(input: &[u8], output: &mut [u8]) -> usize {
    constant_time::encode_slice_ct(input, output)
}
