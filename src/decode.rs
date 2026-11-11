//! Base64 decoding implementations, including zero-alloc in-place decoding.

use crate::alphabet::Config;
use crate::error::DecodeError;
use crate::scalar::decode_scalar;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Decodes Base64 data into a caller-provided mutable slice.
/// Returns the number of bytes written to `output`.
#[inline]
pub fn decode_to_slice(
    input: &[u8],
    output: &mut [u8],
    config: &Config,
) -> Result<usize, DecodeError> {
    let in_len = input.len();
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

    #[cfg(target_arch = "x86_64")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if crate::simd::has_avx512() && input.len() >= 64 {
            let (in_done, out_done) =
                unsafe { crate::simd::decode_avx512(input, output, is_url_safe) };
            if in_done > 0 {
                let rem_in = &input[in_done..];
                let rem_out = &mut output[out_done..];
                let written = decode_to_slice(rem_in, rem_out, config)?;
                return Ok(out_done + written);
            }
        } else if crate::simd::has_avx2() && input.len() >= 32 {
            let (in_done, out_done) =
                unsafe { crate::simd::decode_avx2(input, output, is_url_safe) };
            if in_done > 0 {
                let rem_in = &input[in_done..];
                let rem_out = &mut output[out_done..];
                let written = decode_scalar(rem_in, rem_out, config)?;
                return Ok(out_done + written);
            }
        } else if crate::simd::has_ssse3() && input.len() >= 16 {
            let (in_done, out_done) =
                unsafe { crate::simd::decode_ssse3(input, output, is_url_safe) };
            if in_done > 0 {
                let rem_in = &input[in_done..];
                let rem_out = &mut output[out_done..];
                let written = decode_scalar(rem_in, rem_out, config)?;
                return Ok(out_done + written);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if crate::simd::has_wasm_simd128() && input.len() >= 16 {
            let (in_done, out_done) =
                unsafe { crate::simd::decode_wasm_simd(input, output, is_url_safe) };
            if in_done > 0 {
                let rem_in = &input[in_done..];
                let rem_out = &mut output[out_done..];
                let written = decode_scalar(rem_in, rem_out, config)?;
                return Ok(out_done + written);
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if input.len() >= 16 {
            let (in_done, out_done) =
                unsafe { crate::simd::decode_neon(input, output, is_url_safe) };
            if in_done > 0 {
                let rem_in = &input[in_done..];
                let rem_out = &mut output[out_done..];
                let written = decode_scalar(rem_in, rem_out, config)?;
                return Ok(out_done + written);
            }
        }
    }

    decode_scalar(input, output, config)
}

/// Decodes Base64 data directly **in-place** within the provided mutable buffer.
///
/// Because Base64 decodes 4 ASCII characters into 3 bytes, the decoded output
/// is always smaller than or equal to the input. The read cursor always leads
/// the write cursor, meaning this operation requires **0 additional bytes of memory**.
///
/// Returns a subslice of `buf` containing the decoded bytes.
pub fn decode_in_place<'a>(
    buf: &'a mut [u8],
    config: &Config,
) -> Result<&'a mut [u8], DecodeError> {
    let len = buf.len();
    if len == 0 {
        return Ok(&mut buf[..0]);
    }

    let dec_table = config.decode_table;
    let mut in_idx = 0;
    let mut out_idx = 0;
    let ptr = buf.as_mut_ptr();

    #[cfg(target_arch = "x86_64")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if crate::simd::has_avx512() && len >= 64 {
            let (in_done, out_done) = unsafe {
                let in_slice = core::slice::from_raw_parts(ptr, len);
                let out_slice = core::slice::from_raw_parts_mut(ptr, len);
                crate::simd::decode_avx512(in_slice, out_slice, is_url_safe)
            };
            in_idx = in_done;
            out_idx = out_done;
            if crate::simd::has_avx2() && (len - in_idx) >= 32 {
                let (in_done2, out_done2) = unsafe {
                    let in_slice = core::slice::from_raw_parts(ptr.add(in_idx), len - in_idx);
                    let out_slice =
                        core::slice::from_raw_parts_mut(ptr.add(out_idx), len - out_idx);
                    crate::simd::decode_avx2(in_slice, out_slice, is_url_safe)
                };
                in_idx += in_done2;
                out_idx += out_done2;
            }
        } else if crate::simd::has_avx2() && len >= 32 {
            let (in_done, out_done) = unsafe {
                let in_slice = core::slice::from_raw_parts(ptr, len);
                let out_slice = core::slice::from_raw_parts_mut(ptr, len);
                crate::simd::decode_avx2(in_slice, out_slice, is_url_safe)
            };
            in_idx = in_done;
            out_idx = out_done;
        } else if crate::simd::has_ssse3() && len >= 16 {
            let (in_done, out_done) = unsafe {
                let in_slice = core::slice::from_raw_parts(ptr, len);
                let out_slice = core::slice::from_raw_parts_mut(ptr, len);
                crate::simd::decode_ssse3(in_slice, out_slice, is_url_safe)
            };
            in_idx = in_done;
            out_idx = out_done;
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if crate::simd::has_wasm_simd128() && len >= 16 {
            let (in_done, out_done) = unsafe {
                let in_slice = core::slice::from_raw_parts(ptr, len);
                let out_slice = core::slice::from_raw_parts_mut(ptr, len);
                crate::simd::decode_wasm_simd(in_slice, out_slice, is_url_safe)
            };
            in_idx = in_done;
            out_idx = out_done;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if len >= 16 {
            let (in_done, out_done) = unsafe {
                let in_slice = core::slice::from_raw_parts(ptr, len);
                let out_slice = core::slice::from_raw_parts_mut(ptr, len);
                crate::simd::decode_neon(in_slice, out_slice, is_url_safe)
            };
            in_idx = in_done;
            out_idx = out_done;
        }
    }

    // Fast loop: Read 4 characters, write 3 bytes
    while in_idx + 4 <= len {
        // Read 4 characters into registers before any writes
        let (c0, c1, c2, c3) = unsafe {
            (
                *ptr.add(in_idx),
                *ptr.add(in_idx + 1),
                *ptr.add(in_idx + 2),
                *ptr.add(in_idx + 3),
            )
        };

        let s0 = dec_table[c0 as usize];
        let s1 = dec_table[c1 as usize];
        let s2 = dec_table[c2 as usize];
        let s3 = dec_table[c3 as usize];

        let combined = s0 | s1 | s2 | s3;

        if (combined & 0xC0) == 0 {
            // Valid sextets
            let n = ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
            unsafe {
                *ptr.add(out_idx) = (n >> 16) as u8;
                *ptr.add(out_idx + 1) = (n >> 8) as u8;
                *ptr.add(out_idx + 2) = n as u8;
            }
            in_idx += 4;
            out_idx += 3;
            continue;
        }

        // Padding or error
        if in_idx + 4 == len {
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
                    unsafe {
                        *ptr.add(out_idx) = (s0 << 2) | (s1 >> 4);
                    }
                    out_idx += 1;
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
                    unsafe {
                        *ptr.add(out_idx) = (s0 << 2) | (s1 >> 4);
                        *ptr.add(out_idx + 1) = ((s1 & 0x0F) << 4) | (s2 >> 2);
                    }
                    out_idx += 2;
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
                let n =
                    ((s0 as u32) << 18) | ((s1 as u32) << 12) | ((s2 as u32) << 6) | (s3 as u32);
                unsafe {
                    *ptr.add(out_idx) = (n >> 16) as u8;
                    *ptr.add(out_idx + 1) = (n >> 8) as u8;
                    *ptr.add(out_idx + 2) = n as u8;
                }
                out_idx += 3;
            }
            return Ok(&mut buf[..out_idx]);
        }

        // Return error for invalid byte
        for (i, &c) in [c0, c1, c2, c3].iter().enumerate() {
            let s = dec_table[c as usize];
            if s >= 64 {
                return Err(DecodeError::InvalidByte {
                    offset: in_idx + i,
                    byte: c,
                });
            }
        }
    }

    if in_idx < len {
        let rem = len - in_idx;
        match rem {
            2 => {
                let c0 = unsafe { *ptr.add(in_idx) };
                let c1 = unsafe { *ptr.add(in_idx + 1) };
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
                unsafe {
                    *ptr.add(out_idx) = (s0 << 2) | (s1 >> 4);
                }
                out_idx += 1;
            }
            3 => {
                let c0 = unsafe { *ptr.add(in_idx) };
                let c1 = unsafe { *ptr.add(in_idx + 1) };
                let c2 = unsafe { *ptr.add(in_idx + 2) };
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
                unsafe {
                    *ptr.add(out_idx) = (s0 << 2) | (s1 >> 4);
                    *ptr.add(out_idx + 1) = ((s1 & 0x0F) << 4) | (s2 >> 2);
                }
                out_idx += 2;
            }
            _ => return Err(DecodeError::InvalidLength),
        }
    }

    Ok(&mut buf[..out_idx])
}

/// Decodes Base64 data into a freshly allocated `Vec<u8>`.
///
/// Avoids zero-filling overhead on large allocations by writing decoded raw bytes
/// directly into the allocated spare capacity before setting length.
#[cfg(feature = "alloc")]
pub fn decode(input: impl AsRef<[u8]>, config: &Config) -> Result<Vec<u8>, DecodeError> {
    let bytes = input.as_ref();
    let max_len = (bytes.len() / 4) * 3 + (bytes.len() % 4);
    let mut out = Vec::with_capacity(max_len);
    // SAFETY: We construct a temporary mutable slice over `out`'s allocated buffer,
    // write decoded bytes into it via `decode_to_slice`, and set `out`'s length
    // strictly to the successfully written byte count.
    let written = unsafe {
        let slice = core::slice::from_raw_parts_mut(out.as_mut_ptr(), max_len);
        decode_to_slice(bytes, slice, config)?
    };
    // SAFETY: All `written` bytes were successfully initialized by `decode_to_slice`.
    unsafe {
        out.set_len(written);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::STANDARD;

    #[test]
    fn test_short_invalid_padded_slice_does_not_underflow() {
        let _ = decode(b"!=", &STANDARD);
        let _ = decode(b"=", &STANDARD);
        let mut out = [0u8; 4];
        let _ = decode_to_slice(b"!=", &mut out, &STANDARD);
        let _ = decode_to_slice(b"=", &mut out, &STANDARD);
    }
}
