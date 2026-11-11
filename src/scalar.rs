//! Ultra-fast scalar Base64 encoder and decoder.
//!
//! Employs 64-bit / 32-bit chunking, branchless multi-sextet validation,
//! and unaligned word writes for maximum instruction-level parallelism.

use crate::alphabet::{Config, INVALID_BYTE, PAD_BYTE};
use crate::error::DecodeError;

/// Returns the exact encoded length for an input of `len` bytes.
#[inline(always)]
pub const fn encoded_len(len: usize, padding: bool) -> usize {
    if padding {
        len.div_ceil(3) * 4
    } else {
        let full_chunks = len / 3;
        let rem = len % 3;
        full_chunks * 4
            + match rem {
                0 => 0,
                1 => 2,
                2 => 3,
                _ => unreachable!(),
            }
    }
}

/// Returns the maximum possible decoded length for an input slice.
#[inline(always)]
pub const fn decoded_len_estimate(encoded_len: usize) -> usize {
    (encoded_len / 4) * 3 + (encoded_len % 4)
}

/// Scalar Base64 encoder.
#[inline]
pub fn encode_scalar(input: &[u8], output: &mut [u8], config: &Config) -> usize {
    let enc_table = config.encode_table;
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();

    // Fast path: Process 12 bytes input -> 16 bytes output per iteration
    while in_idx + 12 <= in_len {
        for _ in 0..4 {
            let b0 = input[in_idx] as u32;
            let b1 = input[in_idx + 1] as u32;
            let b2 = input[in_idx + 2] as u32;
            in_idx += 3;

            let n = (b0 << 16) | (b1 << 8) | b2;
            let c0 = enc_table[(n >> 18) as usize];
            let c1 = enc_table[((n >> 12) & 0x3F) as usize];
            let c2 = enc_table[((n >> 6) & 0x3F) as usize];
            let c3 = enc_table[(n & 0x3F) as usize];

            let word = u32::from_ne_bytes([c0, c1, c2, c3]);
            unsafe {
                core::ptr::write_unaligned(output.as_mut_ptr().add(out_idx) as *mut u32, word);
            }
            out_idx += 4;
        }
    }

    // Process remaining 3-byte chunks
    while in_idx + 3 <= in_len {
        let b0 = input[in_idx] as u32;
        let b1 = input[in_idx + 1] as u32;
        let b2 = input[in_idx + 2] as u32;
        in_idx += 3;

        let n = (b0 << 16) | (b1 << 8) | b2;
        let c0 = enc_table[(n >> 18) as usize];
        let c1 = enc_table[((n >> 12) & 0x3F) as usize];
        let c2 = enc_table[((n >> 6) & 0x3F) as usize];
        let c3 = enc_table[(n & 0x3F) as usize];

        let word = u32::from_ne_bytes([c0, c1, c2, c3]);
        unsafe {
            core::ptr::write_unaligned(output.as_mut_ptr().add(out_idx) as *mut u32, word);
        }
        out_idx += 4;
    }

    // Handle tail (1 or 2 leftover bytes)
    let rem = in_len - in_idx;
    if rem == 1 {
        let b0 = input[in_idx] as u32;
        let c0 = enc_table[(b0 >> 2) as usize];
        let c1 = enc_table[((b0 & 0x03) << 4) as usize];
        output[out_idx] = c0;
        output[out_idx + 1] = c1;
        out_idx += 2;
        if config.padding {
            output[out_idx] = b'=';
            output[out_idx + 1] = b'=';
            out_idx += 2;
        }
    } else if rem == 2 {
        let b0 = input[in_idx] as u32;
        let b1 = input[in_idx + 1] as u32;
        let c0 = enc_table[(b0 >> 2) as usize];
        let c1 = enc_table[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize];
        let c2 = enc_table[((b1 & 0x0F) << 2) as usize];
        output[out_idx] = c0;
        output[out_idx + 1] = c1;
        output[out_idx + 2] = c2;
        out_idx += 3;
        if config.padding {
            output[out_idx] = b'=';
            out_idx += 1;
        }
    }

    out_idx
}

/// Scalar Base64 decoder.
/// Decodes `input` directly into `output`, returning the number of decoded bytes.
#[inline]
pub fn decode_scalar(
    input: &[u8],
    output: &mut [u8],
    config: &Config,
) -> Result<usize, DecodeError> {
    let dec_table = config.decode_table;
    let in_len = input.len();
    if in_len == 0 {
        return Ok(0);
    }

    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_ptr = input.as_ptr();
    let out_ptr = output.as_mut_ptr();

    // Fast path: Unroll by 16 characters (12 decoded bytes) per iteration
    while in_idx + 16 <= in_len {
        let mut combined = 0u8;
        let mut sextets = [0u8; 16];

        for (i, slot) in sextets.iter_mut().enumerate() {
            let c = unsafe { *in_ptr.add(in_idx + i) };
            let s = dec_table[c as usize];
            *slot = s;
            combined |= s;
        }

        if (combined & 0xC0) == 0 {
            // All 16 characters valid! Pack into 12 bytes
            for chunk in 0..4 {
                let base = chunk * 4;
                let s0 = sextets[base] as u32;
                let s1 = sextets[base + 1] as u32;
                let s2 = sextets[base + 2] as u32;
                let s3 = sextets[base + 3] as u32;
                let n = (s0 << 18) | (s1 << 12) | (s2 << 6) | s3;

                let out_offset = out_idx + chunk * 3;
                unsafe {
                    *out_ptr.add(out_offset) = (n >> 16) as u8;
                    *out_ptr.add(out_offset + 1) = (n >> 8) as u8;
                    *out_ptr.add(out_offset + 2) = n as u8;
                }
            }
            in_idx += 16;
            out_idx += 12;
            continue;
        }

        // Break to standard 4-byte processing if padding or invalid byte found
        break;
    }

    // 4-character loop
    while in_idx + 4 <= in_len {
        let (c0, c1, c2, c3) = unsafe {
            (
                *in_ptr.add(in_idx),
                *in_ptr.add(in_idx + 1),
                *in_ptr.add(in_idx + 2),
                *in_ptr.add(in_idx + 3),
            )
        };

        let s0 = dec_table[c0 as usize];
        let s1 = dec_table[c1 as usize];
        let s2 = dec_table[c2 as usize];
        let s3 = dec_table[c3 as usize];

        let combined = s0 | s1 | s2 | s3;

        if (combined & 0xC0) == 0 {
            let n = ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
            unsafe {
                *out_ptr.add(out_idx) = (n >> 16) as u8;
                *out_ptr.add(out_idx + 1) = (n >> 8) as u8;
                *out_ptr.add(out_idx + 2) = n as u8;
            }
            in_idx += 4;
            out_idx += 3;
            continue;
        }

        // Slow path: Contains padding or invalid character
        if in_idx + 4 == in_len {
            return decode_tail(input, in_idx, output, out_idx, config);
        }

        for (i, &c) in [c0, c1, c2, c3].iter().enumerate() {
            let s = dec_table[c as usize];
            if s == INVALID_BYTE || s == PAD_BYTE {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx + i,
                    byte: c,
                });
            }
        }
    }

    if in_idx < in_len {
        decode_unpadded_remainder(input, in_idx, output, out_idx, config)
    } else {
        Ok(out_idx)
    }
}

/// Handles the final 4-byte chunk when padding is present.
#[inline(never)]
fn decode_tail(
    input: &[u8],
    in_idx: usize,
    output: &mut [u8],
    out_idx: usize,
    config: &Config,
) -> Result<usize, DecodeError> {
    let dec_table = config.decode_table;
    let c0 = input[in_idx];
    let c1 = input[in_idx + 1];
    let c2 = input[in_idx + 2];
    let c3 = input[in_idx + 3];

    let s0 = dec_table[c0 as usize];
    let s1 = dec_table[c1 as usize];

    if s0 >= 64 {
        return Err(DecodeError::InvalidByte {
            offset: in_idx,
            byte: c0,
        });
    }
    if s1 >= 64 {
        return Err(DecodeError::InvalidByte {
            offset: in_idx + 1,
            byte: c1,
        });
    }

    if c3 == b'=' {
        if c2 == b'=' {
            if (s1 & 0x0F) != 0 {
                return Err(DecodeError::InvalidPadding { offset: in_idx + 1 });
            }
            output[out_idx] = (s0 << 2) | (s1 >> 4);
            Ok(out_idx + 1)
        } else {
            let s2 = dec_table[c2 as usize];
            if s2 >= 64 {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx + 2,
                    byte: c2,
                });
            }
            if (s2 & 0x03) != 0 {
                return Err(DecodeError::InvalidPadding { offset: in_idx + 2 });
            }
            output[out_idx] = (s0 << 2) | (s1 >> 4);
            output[out_idx + 1] = ((s1 & 0x0F) << 4) | (s2 >> 2);
            Ok(out_idx + 2)
        }
    } else {
        let s2 = dec_table[c2 as usize];
        let s3 = dec_table[c3 as usize];
        if s2 >= 64 {
            return Err(DecodeError::InvalidByte {
                offset: in_idx + 2,
                byte: c2,
            });
        }
        if s3 >= 64 {
            return Err(DecodeError::InvalidByte {
                offset: in_idx + 3,
                byte: c3,
            });
        }
        let n = ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
        output[out_idx] = (n >> 16) as u8;
        output[out_idx + 1] = (n >> 8) as u8;
        output[out_idx + 2] = n as u8;
        Ok(out_idx + 3)
    }
}

/// Decodes unpadded leftover characters (e.g. 2 or 3 chars in URL-safe unpadded).
#[inline(never)]
fn decode_unpadded_remainder(
    input: &[u8],
    in_idx: usize,
    output: &mut [u8],
    out_idx: usize,
    config: &Config,
) -> Result<usize, DecodeError> {
    let rem = input.len() - in_idx;
    let dec_table = config.decode_table;

    match rem {
        2 => {
            let c0 = input[in_idx];
            let c1 = input[in_idx + 1];
            let s0 = dec_table[c0 as usize];
            let s1 = dec_table[c1 as usize];
            if s0 >= 64 {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx,
                    byte: c0,
                });
            }
            if s1 >= 64 {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx + 1,
                    byte: c1,
                });
            }
            if (s1 & 0x0F) != 0 {
                return Err(DecodeError::InvalidPadding { offset: in_idx + 1 });
            }
            output[out_idx] = (s0 << 2) | (s1 >> 4);
            Ok(out_idx + 1)
        }
        3 => {
            let c0 = input[in_idx];
            let c1 = input[in_idx + 1];
            let c2 = input[in_idx + 2];
            let s0 = dec_table[c0 as usize];
            let s1 = dec_table[c1 as usize];
            let s2 = dec_table[c2 as usize];
            if s0 >= 64 {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx,
                    byte: c0,
                });
            }
            if s1 >= 64 {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx + 1,
                    byte: c1,
                });
            }
            if s2 >= 64 {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx + 2,
                    byte: c2,
                });
            }
            if (s2 & 0x03) != 0 {
                return Err(DecodeError::InvalidPadding { offset: in_idx + 2 });
            }
            output[out_idx] = (s0 << 2) | (s1 >> 4);
            output[out_idx + 1] = ((s1 & 0x0F) << 4) | (s2 >> 2);
            Ok(out_idx + 2)
        }
        _ => Err(DecodeError::InvalidLength),
    }
}
