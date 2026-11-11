//! Base64 encoding implementations with SIMD acceleration and zero-alloc slices.

use crate::alphabet::Config;
use crate::scalar::{encode_scalar, encoded_len};

#[cfg(target_arch = "x86_64")]
use crate::simd::{encode_avx2, encode_ssse3, has_avx2, has_ssse3};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Encodes `input` bytes into a caller-provided mutable output slice.
/// Returns the number of ASCII bytes written.
#[inline]
pub fn encode_to_slice(input: &[u8], output: &mut [u8], config: &Config) -> usize {
    let req_len = encoded_len(input.len(), config.padding);
    assert!(
        output.len() >= req_len,
        "Destination slice too small: needed {} bytes, got {}",
        req_len,
        output.len()
    );

    #[cfg(target_arch = "x86_64")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if has_avx2() && input.len() >= 24 {
            let (in_done, out_done) = unsafe { encode_avx2(input, output, is_url_safe) };
            let rem_in = &input[in_done..];
            let rem_out = &mut output[out_done..];
            return out_done + encode_scalar(rem_in, rem_out, config);
        } else if has_ssse3() && input.len() >= 12 {
            let (in_done, out_done) = unsafe { encode_ssse3(input, output, is_url_safe) };
            let rem_in = &input[in_done..];
            let rem_out = &mut output[out_done..];
            return out_done + encode_scalar(rem_in, rem_out, config);
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        let is_url_safe = config.encode_table[62] == b'-';
        if input.len() >= 12 {
            let (in_done, out_done) =
                unsafe { crate::simd::encode_neon(input, output, is_url_safe) };
            let rem_in = &input[in_done..];
            let rem_out = &mut output[out_done..];
            return out_done + encode_scalar(rem_in, rem_out, config);
        }
    }

    encode_scalar(input, output, config)
}

/// Encodes `input` into a newly allocated [`String`].
///
/// Avoids zero-filling overhead by directly writing encoded Base64 into uninitialized
/// heap capacity before updating vector length.
#[cfg(feature = "alloc")]
pub fn encode(input: impl AsRef<[u8]>, config: &Config) -> String {
    let bytes = input.as_ref();
    let len = encoded_len(bytes.len(), config.padding);
    let mut vec = Vec::with_capacity(len);
    // SAFETY: We construct a temporary mutable slice over `vec`'s allocated spare capacity,
    // write exactly `len` initialized ASCII bytes into it via `encode_to_slice`, and only then
    // set the vector's length to `written` (which equals `len`). No uninitialized bytes are read.
    let written = unsafe {
        let slice = core::slice::from_raw_parts_mut(vec.as_mut_ptr(), len);
        encode_to_slice(bytes, slice, config)
    };
    debug_assert_eq!(written, len);
    // SAFETY: All `written` bytes were successfully initialized as valid ASCII characters.
    unsafe {
        vec.set_len(written);
        String::from_utf8_unchecked(vec)
    }
}

/// An iterator that encodes input bytes into 4-byte Base64 chunks on the fly.
///
/// Yields fixed-size `[u8; 4]` arrays with **zero heap allocations**, ideal
/// for low-overhead chunked network transmission or streaming iterators.
#[derive(Debug, Clone)]
pub struct Base64ChunkIter<'a> {
    input: &'a [u8],
    config: &'static Config,
    offset: usize,
    finished: bool,
}

impl<'a> Base64ChunkIter<'a> {
    /// Creates a new chunk iterator with the specified configuration.
    #[inline(always)]
    pub const fn new(input: &'a [u8], config: &'static Config) -> Self {
        Self {
            input,
            config,
            offset: 0,
            finished: false,
        }
    }
}

impl<'a> Iterator for Base64ChunkIter<'a> {
    type Item = [u8; 4];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.offset >= self.input.len() {
            return None;
        }

        let remaining = &self.input[self.offset..];
        let mut out = [0u8; 4];

        if remaining.len() >= 3 {
            let written = encode_scalar(&remaining[..3], &mut out, self.config);
            debug_assert_eq!(written, 4);
            self.offset += 3;
            Some(out)
        } else {
            let written = encode_scalar(remaining, &mut out, self.config);
            debug_assert_eq!(written, 4);
            self.offset += remaining.len();
            self.finished = true;
            Some(out)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finished {
            return (0, Some(0));
        }
        let rem_bytes = self.input.len().saturating_sub(self.offset);
        let chunks = if rem_bytes == 0 {
            0
        } else {
            rem_bytes.div_ceil(3)
        };
        (chunks, Some(chunks))
    }
}

impl<'a> ExactSizeIterator for Base64ChunkIter<'a> {}

/// Creates a streaming iterator over 4-byte Base64 chunks using the standard alphabet.
#[inline(always)]
pub fn encode_iter<'a>(input: &'a [u8]) -> Base64ChunkIter<'a> {
    Base64ChunkIter::new(input, &crate::alphabet::Config::STANDARD)
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_encode_chunk_iter() {
        let input = b"Hello World!";
        let chunks: Vec<[u8; 4]> = encode_iter(input).collect();
        let flattened: Vec<u8> = chunks.into_iter().flatten().collect();
        assert_eq!(&flattened[..], b"SGVsbG8gV29ybGQh");
    }

    #[test]
    fn test_encode_string() {
        let input = b"Performance test without double write";
        let encoded = encode(input, &Config::STANDARD);
        assert_eq!(encoded, "UGVyZm9ybWFuY2UgdGVzdCB3aXRob3V0IGRvdWJsZSB3cml0ZQ==");
    }
}
