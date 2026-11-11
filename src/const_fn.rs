//! Compile-time (`const fn`) Base64 encoding and decoding.
//!
//! Enables zero-overhead Base64 transformations at build time without macros
//! or external build scripts.

/// Calculates the exact decoded length for a Base64 byte slice at compile time.
#[inline(always)]
pub const fn decoded_len(input: &[u8]) -> usize {
    let len = input.len();
    if len == 0 {
        return 0;
    }
    if len >= 2 && input[len - 1] == b'=' && input[len - 2] == b'=' {
        return (len / 4) * 3 - 2;
    }
    if input[len - 1] == b'=' {
        return (len / 4) * 3 - 1;
    }
    let full = (len / 4) * 3;
    match len % 4 {
        0 => full,
        2 => full + 1,
        3 => full + 2,
        _ => full,
    }
}

/// Encodes an input byte slice into a fixed-size byte array at compile time.
pub const fn encode_to_array<const OUT: usize>(
    input: &[u8],
    table: &'static [u8; 64],
    padding: bool,
) -> [u8; OUT] {
    let mut out = [0u8; OUT];
    let in_len = input.len();
    let mut in_idx = 0;
    let mut out_idx = 0;

    while in_idx + 3 <= in_len {
        let b0 = input[in_idx] as u32;
        let b1 = input[in_idx + 1] as u32;
        let b2 = input[in_idx + 2] as u32;
        in_idx += 3;

        let n = (b0 << 16) | (b1 << 8) | b2;
        out[out_idx] = table[(n >> 18) as usize];
        out[out_idx + 1] = table[((n >> 12) & 0x3F) as usize];
        out[out_idx + 2] = table[((n >> 6) & 0x3F) as usize];
        out[out_idx + 3] = table[(n & 0x3F) as usize];
        out_idx += 4;
    }

    let rem = in_len - in_idx;
    if rem == 1 {
        let b0 = input[in_idx] as u32;
        out[out_idx] = table[(b0 >> 2) as usize];
        out[out_idx + 1] = table[((b0 & 0x03) << 4) as usize];
        out_idx += 2;
        if padding {
            out[out_idx] = b'=';
            out[out_idx + 1] = b'=';
        }
    } else if rem == 2 {
        let b0 = input[in_idx] as u32;
        let b1 = input[in_idx + 1] as u32;
        out[out_idx] = table[(b0 >> 2) as usize];
        out[out_idx + 1] = table[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        out[out_idx + 2] = table[((b1 & 0x0F) << 2) as usize];
        out_idx += 3;
        if padding {
            out[out_idx] = b'=';
        }
    }

    out
}

/// Decodes an input Base64 byte slice into a fixed-size byte array at compile time.
pub const fn decode_to_array<const OUT: usize>(
    input: &[u8],
    table: &'static [u8; 256],
) -> [u8; OUT] {
    let mut out = [0u8; OUT];
    let in_len = input.len();
    let mut in_idx = 0;
    let mut out_idx = 0;

    while in_idx + 4 <= in_len && out_idx + 3 <= OUT {
        let c0 = input[in_idx];
        let c1 = input[in_idx + 1];
        let c2 = input[in_idx + 2];
        let c3 = input[in_idx + 3];

        let s0 = table[c0 as usize];
        let s1 = table[c1 as usize];
        let s2 = table[c2 as usize];
        let s3 = table[c3 as usize];

        if (s0 | s1 | s2 | s3) < 64 {
            let n = ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
            out[out_idx] = (n >> 16) as u8;
            out[out_idx + 1] = (n >> 8) as u8;
            out[out_idx + 2] = n as u8;
            in_idx += 4;
            out_idx += 3;
        } else {
            // Padding handling
            if c3 == b'=' {
                if c2 == b'=' {
                    if out_idx < OUT {
                        out[out_idx] = (s0 << 2) | (s1 >> 4);
                    }
                } else if out_idx + 1 < OUT {
                    out[out_idx] = (s0 << 2) | (s1 >> 4);
                    out[out_idx + 1] = ((s1 & 0x0F) << 4) | (s2 >> 2);
                }
            }
            return out;
        }
    }

    if in_idx + 2 <= in_len && out_idx < OUT {
        let s0 = table[input[in_idx] as usize];
        let s1 = table[input[in_idx + 1] as usize];
        if s0 < 64 && s1 < 64 {
            out[out_idx] = (s0 << 2) | (s1 >> 4);
            out_idx += 1;
            if in_idx + 3 <= in_len && out_idx < OUT {
                let s2 = table[input[in_idx + 2] as usize];
                if s2 < 64 {
                    out[out_idx] = ((s1 & 0x0F) << 4) | (s2 >> 2);
                }
            }
        }
    }

    out
}

/// Helper macro to encode a literal string or byte slice at compile time.
///
/// # Example
/// ```rust
/// use hyper64::encode_const;
/// const HELLO_B64: &[u8] = &encode_const!(b"Hello World");
/// assert_eq!(HELLO_B64, b"SGVsbG8gV29ybGQ=");
/// ```
#[macro_export]
macro_rules! encode_const {
    ($bytes:expr) => {{
        const IN: &[u8] = $bytes;
        const OUT_LEN: usize = $crate::encoded_len(IN.len(), true);
        const RES: [u8; OUT_LEN] =
            $crate::const_fn::encode_to_array::<OUT_LEN>(IN, $crate::STANDARD_ENCODE_TABLE, true);
        RES
    }};
}

/// Helper macro to decode a Base64 literal at compile time.
///
/// # Example
/// ```rust
/// use hyper64::decode_const;
/// const HELLO: [u8; 11] = decode_const!(b"SGVsbG8gV29ybGQ=");
/// assert_eq!(&HELLO, b"Hello World");
/// ```
#[macro_export]
macro_rules! decode_const {
    ($bytes:expr) => {{
        const IN: &[u8] = $bytes;
        const OUT_LEN: usize = $crate::const_fn::decoded_len(IN);
        const RES: [u8; OUT_LEN] =
            $crate::const_fn::decode_to_array::<OUT_LEN>(IN, &$crate::STANDARD_DECODE_TABLE);
        RES
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::STANDARD_ENCODE_TABLE;

    #[test]
    fn test_compile_time_encode() {
        const RES: [u8; 16] = encode_to_array::<16>(b"Hello World", STANDARD_ENCODE_TABLE, true);
        assert_eq!(&RES, b"SGVsbG8gV29ybGQ=");

        const MACRO_RES: [u8; 16] = encode_const!(b"Hello World");
        assert_eq!(&MACRO_RES, b"SGVsbG8gV29ybGQ=");
    }

    #[test]
    fn test_compile_time_decode() {
        const DECODED: [u8; 11] = decode_const!(b"SGVsbG8gV29ybGQ=");
        assert_eq!(&DECODED, b"Hello World");

        const UNPADDED: [u8; 11] = decode_const!(b"SGVsbG8gV29ybGQ");
        assert_eq!(&UNPADDED, b"Hello World");
    }
}
