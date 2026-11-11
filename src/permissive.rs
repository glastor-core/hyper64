//! Permissive and PEM/MIME Base64 decoder.
//!
//! Handles real-world inputs with embedded whitespace (newlines, carriage returns,
//! spaces, indentation) such as MIME emails, PEM certificates, and SSH keys
//! in-place without prior memory sanitization or intermediate heap allocations.

use crate::alphabet::Config;
use crate::error::DecodeError;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Represents a parsed and decoded PEM block.
#[derive(Debug, PartialEq, Eq)]
pub struct PemBlock<'a> {
    /// The PEM block label, e.g. "CERTIFICATE" or "RSA PRIVATE KEY".
    pub label: &'a str,
    /// The in-place decoded binary payload (DER/ASN.1).
    pub contents: &'a mut [u8],
}

#[inline(always)]
fn has_byte_le_32(v: u64) -> bool {
    ((v.wrapping_sub(0x2121212121212121)) & !v & 0x8080808080808080) != 0
}

/// Decodes Base64 data with whitespace tolerance directly **in-place**.
///
/// Skips ASCII whitespace (`\r`, `\n`, `' '`, `\t`) on the fly with 0 heap allocations.
/// Returns a mutable subslice of `buf` containing the decoded bytes.
pub fn decode_permissive_in_place<'a>(
    buf: &'a mut [u8],
    config: &Config,
) -> Result<&'a mut [u8], DecodeError> {
    let len = buf.len();
    if len == 0 {
        return Ok(&mut buf[..0]);
    }

    let dec_table = config.decode_table;
    let mut r = 0;
    let mut w = 0;
    let ptr = buf.as_mut_ptr();

    while r < len {
        // Fast-path: Check 8-byte block for contiguous non-whitespace valid Base64 symbols
        while r + 8 <= len {
            let b = unsafe { core::ptr::read_unaligned(ptr.add(r) as *const [u8; 8]) };
            let chunk = u64::from_ne_bytes(b);
            if has_byte_le_32(chunk) {
                break;
            }

            let s0 = dec_table[b[0] as usize];
            let s1 = dec_table[b[1] as usize];
            let s2 = dec_table[b[2] as usize];
            let s3 = dec_table[b[3] as usize];
            let s4 = dec_table[b[4] as usize];
            let s5 = dec_table[b[5] as usize];
            let s6 = dec_table[b[6] as usize];
            let s7 = dec_table[b[7] as usize];

            if (s0 | s1 | s2 | s3 | s4 | s5 | s6 | s7) >= 64 {
                // Contains padding '=' or invalid character; fallback to scalar parser
                break;
            }

            let n0 = ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
            let n1 = ((s4 as u32) << 18) | ((s5 as u32) << 12) | ((s6 as u32) << 6) | (s7 as u32);

            unsafe {
                *ptr.add(w) = (n0 >> 16) as u8;
                *ptr.add(w + 1) = (n0 >> 8) as u8;
                *ptr.add(w + 2) = n0 as u8;
                *ptr.add(w + 3) = (n1 >> 16) as u8;
                *ptr.add(w + 4) = (n1 >> 8) as u8;
                *ptr.add(w + 5) = n1 as u8;
            }

            r += 8;
            w += 6;
        }

        if r >= len {
            break;
        }

        let mut quad = [0u8; 4];
        let mut quad_len = 0;

        while r < len && quad_len < 4 {
            let b = unsafe { *ptr.add(r) };
            r += 1;
            if b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' {
                continue;
            }
            quad[quad_len] = b;
            quad_len += 1;
        }

        if quad_len == 0 {
            break;
        }

        if quad_len == 4 {
            let c0 = quad[0];
            let c1 = quad[1];
            let c2 = quad[2];
            let c3 = quad[3];

            let s0 = dec_table[c0 as usize];
            let s1 = dec_table[c1 as usize];
            let s2 = dec_table[c2 as usize];
            let s3 = dec_table[c3 as usize];

            if (s0 | s1 | s2 | s3) < 64 {
                let n =
                    ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
                unsafe {
                    *ptr.add(w) = (n >> 16) as u8;
                    *ptr.add(w + 1) = (n >> 8) as u8;
                    *ptr.add(w + 2) = n as u8;
                }
                w += 3;
                continue;
            }

            // Padding or invalid character
            if c3 == b'=' {
                if c2 == b'=' {
                    if s0 >= 64 || s1 >= 64 || (s1 & 0x0F) != 0 {
                        return Err(DecodeError::InvalidPadding { offset: r });
                    }
                    unsafe {
                        *ptr.add(w) = (s0 << 2) | (s1 >> 4);
                    }
                    w += 1;
                } else {
                    if s0 >= 64 || s1 >= 64 || s2 >= 64 || (s2 & 0x03) != 0 {
                        return Err(DecodeError::InvalidPadding { offset: r });
                    }
                    unsafe {
                        *ptr.add(w) = (s0 << 2) | (s1 >> 4);
                        *ptr.add(w + 1) = ((s1 & 0x0F) << 4) | (s2 >> 2);
                    }
                    w += 2;
                }

                // Verify that only whitespace remains after padding
                while r < len {
                    let b = unsafe { *ptr.add(r) };
                    r += 1;
                    if b != b' ' && b != b'\t' && b != b'\r' && b != b'\n' {
                        return Err(DecodeError::InvalidByte {
                            offset: r - 1,
                            byte: b,
                        });
                    }
                }
                break;
            }

            for (idx, &c) in [c0, c1, c2, c3].iter().enumerate() {
                if dec_table[c as usize] >= 64 {
                    return Err(DecodeError::InvalidByte {
                        offset: r - 4 + idx,
                        byte: c,
                    });
                }
            }
        } else {
            // Unpadded tail (2 or 3 chars)
            match quad_len {
                2 => {
                    let s0 = dec_table[quad[0] as usize];
                    let s1 = dec_table[quad[1] as usize];
                    if s0 >= 64 || s1 >= 64 || (s1 & 0x0F) != 0 {
                        return Err(DecodeError::InvalidPadding { offset: r });
                    }
                    unsafe {
                        *ptr.add(w) = (s0 << 2) | (s1 >> 4);
                    }
                    w += 1;
                }
                3 => {
                    let s0 = dec_table[quad[0] as usize];
                    let s1 = dec_table[quad[1] as usize];
                    let s2 = dec_table[quad[2] as usize];
                    if s0 >= 64 || s1 >= 64 || s2 >= 64 || (s2 & 0x03) != 0 {
                        return Err(DecodeError::InvalidPadding { offset: r });
                    }
                    unsafe {
                        *ptr.add(w) = (s0 << 2) | (s1 >> 4);
                        *ptr.add(w + 1) = ((s1 & 0x0F) << 4) | (s2 >> 2);
                    }
                    w += 2;
                }
                _ => return Err(DecodeError::InvalidLength),
            }
        }
    }

    Ok(&mut buf[..w])
}

/// Decodes permissive Base64 input into a freshly allocated `Vec<u8>`.
#[cfg(feature = "alloc")]
pub fn decode_permissive(input: impl AsRef<[u8]>, config: &Config) -> Result<Vec<u8>, DecodeError> {
    let mut bytes = input.as_ref().to_vec();
    let written = decode_permissive_in_place(&mut bytes, config)?.len();
    bytes.truncate(written);
    Ok(bytes)
}

/// Decodes a PEM-encoded block (such as an X.509 certificate or private key) directly **in-place**.
///
/// Automatically extracts the PEM header label and decodes the inner payload with 0 allocations.
pub fn decode_pem_in_place<'a>(buf: &'a mut [u8]) -> Result<PemBlock<'a>, DecodeError> {
    const BEGIN_PREFIX: &[u8] = b"-----BEGIN ";
    const BANNER_SUFFIX: &[u8] = b"-----";
    const END_PREFIX: &[u8] = b"-----END ";

    // 1. Locate "-----BEGIN "
    let begin_pos = buf
        .windows(BEGIN_PREFIX.len())
        .position(|window| window == BEGIN_PREFIX)
        .ok_or(DecodeError::InvalidByte { offset: 0, byte: 0 })?;

    let label_start = begin_pos + BEGIN_PREFIX.len();
    let label_end = buf[label_start..]
        .windows(BANNER_SUFFIX.len())
        .position(|window| window == BANNER_SUFFIX)
        .map(|pos| label_start + pos)
        .ok_or(DecodeError::InvalidByte {
            offset: label_start,
            byte: 0,
        })?;

    let content_start = label_end + BANNER_SUFFIX.len();

    // 2. Locate "-----END "
    let end_pos = buf[content_start..]
        .windows(END_PREFIX.len())
        .position(|window| window == END_PREFIX)
        .map(|pos| content_start + pos)
        .ok_or(DecodeError::InvalidByte {
            offset: content_start,
            byte: 0,
        })?;

    // Split buf at content_start into header and rest
    let (head, rest) = buf.split_at_mut(content_start);

    // Extract label as &str from head
    let label_bytes = &head[label_start..label_end];
    let label = core::str::from_utf8(label_bytes).map_err(|_| DecodeError::InvalidByte {
        offset: label_start,
        byte: label_bytes[0],
    })?;

    let content_len = end_pos - content_start;
    let content_slice = &mut rest[..content_len];

    let decoded = decode_permissive_in_place(content_slice, &Config::STANDARD)?;

    Ok(PemBlock {
        label,
        contents: decoded,
    })
}

/// Decodes a PEM-encoded certificate or key into a label string and payload vector.
#[cfg(feature = "alloc")]
pub fn decode_pem(input: impl AsRef<[u8]>) -> Result<(String, Vec<u8>), DecodeError> {
    let mut data = input.as_ref().to_vec();
    let block = decode_pem_in_place(&mut data)?;
    let label = String::from(block.label);
    let bytes = block.contents.to_vec();
    Ok((label, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_whitespace() {
        let mut dirty = *b"  SGVs\r\nbG8g\n  V29y\tbGQ= \r\n";
        let decoded = decode_permissive_in_place(&mut dirty, &Config::STANDARD).unwrap();
        assert_eq!(decoded, b"Hello World");
    }

    #[test]
    fn test_pem_in_place() {
        let mut pem =
            *b"-----BEGIN CERTIFICATE-----\r\nSGVsbG8gV29ybGQ=\r\n-----END CERTIFICATE-----\r\n";
        let block = decode_pem_in_place(&mut pem).unwrap();
        assert_eq!(block.label, "CERTIFICATE");
        assert_eq!(block.contents, b"Hello World");
    }
}
