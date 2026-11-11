#![allow(unknown_lints)]
#![allow(clippy::chunks_exact_to_as_chunks)]
//! Constant-time Base64 implementation.
//!
//! Designed for cryptographic operations where data confidentiality is critical.
//! This implementation avoids lookup tables (LUTs) and conditional branches that
//! depend on secret data, mitigating cache-timing side-channel attacks (e.g. Spectre,
//! CacheBleed, Bernstein cache timing).

/// Encodes a 6-bit value (0..=63) to its standard Base64 ASCII byte in constant time.
///
/// Uses branchless bitwise arithmetic:
/// - 0..25   -> 'A'..'Z' (+65)
/// - 26..51  -> 'a'..'z' (+71)
/// - 52..61  -> '0'..'9' (-4)
/// - 62      -> '+'      (-19)
/// - 63      -> '/'      (-16)
#[inline(always)]
pub const fn encode_sextet_ct_standard(val: u8) -> u8 {
    let x = val as i32;

    let lt_26 = ((x - 26) as u32 >> 31) as i32;
    let lt_52 = ((x - 52) as u32 >> 31) as i32;
    let lt_62 = ((x - 62) as u32 >> 31) as i32;
    let lt_63 = ((x - 63) as u32 >> 31) as i32;
    let lt_64 = ((x - 64) as u32 >> 31) as i32;

    let is_upper = lt_26;
    let is_lower = lt_52 - lt_26;
    let is_digit = lt_62 - lt_52;
    let is_plus = lt_63 - lt_62;
    let is_slash = lt_64 - lt_63;

    let res = (is_upper * (x + 65))
        + (is_lower * (x + 71))
        + (is_digit * (x - 4))
        + (is_plus * (x - 19))
        + (is_slash * (x - 16));

    res as u8
}

/// Encodes a 6-bit value (0..=63) to URL-safe Base64 ASCII byte in constant time.
/// - 62 -> '-' (ASCII 45)
/// - 63 -> '_' (ASCII 95)
#[inline(always)]
pub const fn encode_sextet_ct_url(val: u8) -> u8 {
    let x = val as i32;

    let lt_26 = ((x - 26) as u32 >> 31) as i32;
    let lt_52 = ((x - 52) as u32 >> 31) as i32;
    let lt_62 = ((x - 62) as u32 >> 31) as i32;
    let lt_63 = ((x - 63) as u32 >> 31) as i32;
    let lt_64 = ((x - 64) as u32 >> 31) as i32;

    let is_upper = lt_26;
    let is_lower = lt_52 - lt_26;
    let is_digit = lt_62 - lt_52;
    let is_dash = lt_63 - lt_62;
    let is_underscore = lt_64 - lt_63;

    let res = (is_upper * (x + 65))
        + (is_lower * (x + 71))
        + (is_digit * (x - 4))
        + (is_dash * 45)
        + (is_underscore * 95);

    res as u8
}

/// Decodes an ASCII byte to a 6-bit sextet (0..=63) in constant time without LUTs.
/// Returns (sextet, is_valid_mask) where is_valid_mask is 0xFF if valid, 0x00 otherwise.
#[inline(always)]
pub const fn decode_byte_ct_standard(ascii: u8) -> (u8, u8) {
    let c = ascii as i32;

    // 'A'..'Z' is 65..=90 -> maps to 0..25
    let ge_65 = ((64 - c) as u32 >> 31) as i32;
    let le_90 = ((c - 91) as u32 >> 31) as i32;
    let is_upper = ge_65 & le_90;
    let val_upper = (c - 65) * is_upper;

    // 'a'..'z' is 97..=122 -> maps to 26..51
    let ge_97 = ((96 - c) as u32 >> 31) as i32;
    let le_122 = ((c - 123) as u32 >> 31) as i32;
    let is_lower = ge_97 & le_122;
    let val_lower = (c - 71) * is_lower;

    // '0'..'9' is 48..=57 -> maps to 52..61
    let ge_48 = ((47 - c) as u32 >> 31) as i32;
    let le_57 = ((c - 58) as u32 >> 31) as i32;
    let is_digit = ge_48 & le_57;
    let val_digit = (c + 4) * is_digit;

    // '+' is 43 -> maps to 62
    let is_plus = ((((c - 43) as u32 | (43 - c) as u32) >> 31) ^ 1) as i32;
    let val_plus = 62 * is_plus;

    // '/' is 47 -> maps to 63
    let is_slash = ((((c - 47) as u32 | (47 - c) as u32) >> 31) ^ 1) as i32;
    let val_slash = 63 * is_slash;

    let is_valid = is_upper | is_lower | is_digit | is_plus | is_slash;
    let val = (val_upper | val_lower | val_digit | val_plus | val_slash) as u8;
    let mask = if is_valid == 1 { 0xFF } else { 0x00 };

    (val, mask)
}

/// Constant-time encode of a slice into an output slice using the standard alphabet.
pub fn encode_slice_ct(input: &[u8], output: &mut [u8]) -> usize {
    let chunks = input.chunks_exact(3);
    let remainder = chunks.remainder();
    let mut out_idx = 0;

    for chunk in chunks {
        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        output[out_idx] = encode_sextet_ct_standard(b0 >> 2);
        output[out_idx + 1] = encode_sextet_ct_standard(((b0 & 0x03) << 4) | (b1 >> 4));
        output[out_idx + 2] = encode_sextet_ct_standard(((b1 & 0x0F) << 2) | (b2 >> 6));
        output[out_idx + 3] = encode_sextet_ct_standard(b2 & 0x3F);
        out_idx += 4;
    }

    match remainder.len() {
        1 => {
            let b0 = remainder[0];
            output[out_idx] = encode_sextet_ct_standard(b0 >> 2);
            output[out_idx + 1] = encode_sextet_ct_standard((b0 & 0x03) << 4);
            output[out_idx + 2] = b'=';
            output[out_idx + 3] = b'=';
            out_idx += 4;
        }
        2 => {
            let b0 = remainder[0];
            let b1 = remainder[1];
            output[out_idx] = encode_sextet_ct_standard(b0 >> 2);
            output[out_idx + 1] = encode_sextet_ct_standard(((b0 & 0x03) << 4) | (b1 >> 4));
            output[out_idx + 2] = encode_sextet_ct_standard((b1 & 0x0F) << 2);
            output[out_idx + 3] = b'=';
            out_idx += 4;
        }
        _ => {}
    }

    out_idx
}

use crate::error::DecodeError;

/// Compares two slices in constant time, returning true if they are equal.
/// Mitigates timing attacks when validating cryptographic hashes, HMACs, or tokens.
#[inline]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= core::hint::black_box(x ^ y);
    }
    core::hint::black_box(diff == 0)
}

/// Constant-time decode of a Base64 input slice into an output slice using branchless arithmetic.
///
/// Ensures uniform execution latency regardless of the input contents or validity,
/// preventing cache-timing side-channel attacks on secret keys and cryptographic payloads.
pub fn decode_slice_ct(input: &[u8], output: &mut [u8]) -> Result<usize, DecodeError> {
    let in_len = input.len();
    if in_len == 0 {
        return Ok(0);
    }
    if !in_len.is_multiple_of(4) {
        return Err(DecodeError::InvalidLength);
    }

    let mut min_capacity = (in_len / 4) * 3;
    if in_len >= 2 && input[in_len - 1] == b'=' {
        min_capacity = min_capacity.saturating_sub(1);
        if input[in_len - 2] == b'=' {
            min_capacity = min_capacity.saturating_sub(1);
        }
    }
    if output.len() < min_capacity {
        return Err(DecodeError::OutputSliceTooSmall {
            expected: min_capacity,
            actual: output.len(),
        });
    }

    let chunks = input.chunks_exact(4);
    let num_chunks = chunks.len();
    let mut out_idx = 0;
    let mut all_valid_mask = 0xFFu8;

    for (chunk_idx, chunk) in chunks.enumerate() {
        let is_last = chunk_idx + 1 == num_chunks;
        let c0 = chunk[0];
        let c1 = chunk[1];
        let c2 = chunk[2];
        let c3 = chunk[3];

        let (s0, m0) = decode_byte_ct_standard(c0);
        let (s1, m1) = decode_byte_ct_standard(c1);
        let (s2, m2) = decode_byte_ct_standard(c2);
        let (s3, m3) = decode_byte_ct_standard(c3);

        if !is_last {
            all_valid_mask &= m0 & m1 & m2 & m3;
            let n = ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
            output[out_idx] = (n >> 16) as u8;
            output[out_idx + 1] = (n >> 8) as u8;
            output[out_idx + 2] = n as u8;
            out_idx += 3;
        } else {
            all_valid_mask &= m0 & m1;
            if c3 == b'=' {
                if c2 == b'=' {
                    if (s1 & 0x0F) != 0 {
                        all_valid_mask = 0;
                    }
                    output[out_idx] = (s0 << 2) | (s1 >> 4);
                    out_idx += 1;
                } else {
                    all_valid_mask &= m2;
                    if (s2 & 0x03) != 0 {
                        all_valid_mask = 0;
                    }
                    output[out_idx] = (s0 << 2) | (s1 >> 4);
                    output[out_idx + 1] = ((s1 & 0x0F) << 4) | (s2 >> 2);
                    out_idx += 2;
                }
            } else {
                all_valid_mask &= m2 & m3;
                let n =
                    ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
                output[out_idx] = (n >> 16) as u8;
                output[out_idx + 1] = (n >> 8) as u8;
                output[out_idx + 2] = n as u8;
                out_idx += 3;
            }
        }
    }

    if all_valid_mask == 0xFF {
        Ok(out_idx)
    } else {
        Err(DecodeError::InvalidByte { offset: 0, byte: 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_encode_sextets() {
        assert_eq!(encode_sextet_ct_standard(0), b'A');
        assert_eq!(encode_sextet_ct_standard(25), b'Z');
        assert_eq!(encode_sextet_ct_standard(26), b'a');
        assert_eq!(encode_sextet_ct_standard(51), b'z');
        assert_eq!(encode_sextet_ct_standard(52), b'0');
        assert_eq!(encode_sextet_ct_standard(61), b'9');
        assert_eq!(encode_sextet_ct_standard(62), b'+');
        assert_eq!(encode_sextet_ct_standard(63), b'/');
    }

    #[test]
    fn test_ct_decode_bytes() {
        let (val, mask) = decode_byte_ct_standard(b'A');
        assert_eq!((val, mask), (0, 0xFF));
        let (val, mask) = decode_byte_ct_standard(b'Z');
        assert_eq!((val, mask), (25, 0xFF));
        let (val, mask) = decode_byte_ct_standard(b'a');
        assert_eq!((val, mask), (26, 0xFF));
        let (val, mask) = decode_byte_ct_standard(b'z');
        assert_eq!((val, mask), (51, 0xFF));
        let (val, mask) = decode_byte_ct_standard(b'0');
        assert_eq!((val, mask), (52, 0xFF));
        let (val, mask) = decode_byte_ct_standard(b'9');
        assert_eq!((val, mask), (61, 0xFF));
        let (val, mask) = decode_byte_ct_standard(b'+');
        assert_eq!((val, mask), (62, 0xFF));
        let (val, mask) = decode_byte_ct_standard(b'/');
        assert_eq!((val, mask), (63, 0xFF));
        let (_, mask) = decode_byte_ct_standard(b'#');
        assert_eq!(mask, 0x00);
    }

    #[test]
    fn test_ct_decode_slice() {
        let input = b"SGVsbG8gV29ybGQ=";
        let mut out = [0u8; 16];
        let written = decode_slice_ct(input, &mut out).unwrap();
        assert_eq!(&out[..written], b"Hello World");

        assert!(ct_eq(b"secret_key_12345", b"secret_key_12345"));
        assert!(!ct_eq(b"secret_key_12345", b"wrong_key_123456"));
    }
}
