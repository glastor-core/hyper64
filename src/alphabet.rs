//! Base64 Alphabet definitions and lookup tables.

/// Sentinel value indicating an invalid Base64 character in the decode table.
pub const INVALID_BYTE: u8 = 0xFF;
/// Sentinel value indicating a padding character (`=`) in the decode table.
pub const PAD_BYTE: u8 = 0xFE;

/// Standard Base64 alphabet characters (RFC 4648 §4).
pub const STANDARD_ENCODE_TABLE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// URL-safe and filename-safe Base64 alphabet characters (RFC 4648 §5).
pub const URL_SAFE_ENCODE_TABLE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Generates a decode table for a 64-character alphabet at compile time.
pub const fn make_decode_table(encode_table: &[u8; 64], pad: Option<u8>) -> [u8; 256] {
    let mut table = [INVALID_BYTE; 256];
    let mut i = 0;
    while i < 64 {
        table[encode_table[i] as usize] = i as u8;
        i += 1;
    }
    if let Some(pad_char) = pad {
        table[pad_char as usize] = PAD_BYTE;
    }
    table
}

/// Precomputed decode table for standard Base64 with '=' padding.
pub static STANDARD_DECODE_TABLE: [u8; 256] = make_decode_table(STANDARD_ENCODE_TABLE, Some(b'='));

/// Precomputed decode table for URL-safe Base64 with '=' padding.
pub static URL_SAFE_DECODE_TABLE: [u8; 256] = make_decode_table(URL_SAFE_ENCODE_TABLE, Some(b'='));

/// Precomputed decode table for URL-safe Base64 without padding.
pub static URL_SAFE_NO_PAD_DECODE_TABLE: [u8; 256] = make_decode_table(URL_SAFE_ENCODE_TABLE, None);

/// Configuration profile for Base64 encoding/decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub encode_table: &'static [u8; 64],
    pub decode_table: &'static [u8; 256],
    pub padding: bool,
}

impl Config {
    /// Standard Base64 profile with '=' padding.
    pub const STANDARD: Self = Self {
        encode_table: STANDARD_ENCODE_TABLE,
        decode_table: &STANDARD_DECODE_TABLE,
        padding: true,
    };

    /// Standard Base64 profile without padding.
    pub const STANDARD_NO_PAD: Self = Self {
        encode_table: STANDARD_ENCODE_TABLE,
        decode_table: &STANDARD_DECODE_TABLE,
        padding: false,
    };

    /// URL-Safe Base64 profile with '=' padding.
    pub const URL_SAFE: Self = Self {
        encode_table: URL_SAFE_ENCODE_TABLE,
        decode_table: &URL_SAFE_DECODE_TABLE,
        padding: true,
    };

    /// URL-Safe Base64 profile without padding.
    pub const URL_SAFE_NO_PAD: Self = Self {
        encode_table: URL_SAFE_ENCODE_TABLE,
        decode_table: &URL_SAFE_NO_PAD_DECODE_TABLE,
        padding: false,
    };
}
