//! Vectorized SIMD implementations for x86_64 (AVX2).

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Safely loads exactly 12 bytes into a 128-bit SIMD register without reading past byte 12.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn load_12_bytes(ptr: *const u8) -> __m128i {
    unsafe {
        let b64 = core::ptr::read_unaligned(ptr as *const u64);
        let b32 = core::ptr::read_unaligned(ptr.add(8) as *const u32);
        _mm_set_epi32(0, b32 as i32, (b64 >> 32) as i32, b64 as i32)
    }
}

/// Encodes 24 bytes to 32 Base64 ASCII bytes using AVX2.
/// Returns (input_bytes_consumed, output_bytes_written).
///
/// # Safety
/// Caller must ensure the host CPU supports AVX2, and `output` has at least 32 bytes of capacity.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn encode_avx2(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();

    // Each AVX2 iteration processes exactly 24 input bytes -> 32 output bytes
    while in_idx + 24 <= in_len {
        unsafe {
            let in_lo = load_12_bytes(input.as_ptr().add(in_idx));
            let in_hi = load_12_bytes(input.as_ptr().add(in_idx + 12));
            let in256 = _mm256_set_m128i(in_hi, in_lo);

            // Shuffle 12 bytes in each 128-bit lane into 4 32-bit integers
            let shuffle_mask = _mm256_setr_epi8(
                2, 1, 0, -1, 5, 4, 3, -1, 8, 7, 6, -1, 11, 10, 9, -1, 2, 1, 0, -1, 5, 4, 3, -1, 8,
                7, 6, -1, 11, 10, 9, -1,
            );
            let shuffled = _mm256_shuffle_epi8(in256, shuffle_mask);

            // Extract 6-bit sextets using bit manipulation
            let t0 = _mm256_and_si256(shuffled, _mm256_set1_epi32(0x00FC0000));
            let t0 = _mm256_srli_epi32(t0, 18);

            let t1 = _mm256_and_si256(shuffled, _mm256_set1_epi32(0x0003F000));
            let t1 = _mm256_srli_epi32(t1, 12);

            let t2 = _mm256_and_si256(shuffled, _mm256_set1_epi32(0x00000FC0));
            let t2 = _mm256_srli_epi32(t2, 6);

            let t3 = _mm256_and_si256(shuffled, _mm256_set1_epi32(0x0000003F));

            let indices = _mm256_or_si256(
                _mm256_or_si256(t0, _mm256_slli_epi32(t1, 8)),
                _mm256_or_si256(_mm256_slli_epi32(t2, 16), _mm256_slli_epi32(t3, 24)),
            );

            let ascii = translate_indices_avx2(indices, url_safe);
            _mm256_storeu_si256(output.as_mut_ptr().add(out_idx) as *mut __m256i, ascii);
        }

        in_idx += 24;
        out_idx += 32;
    }

    (in_idx, out_idx)
}

/// Translates 32 6-bit indices (0..63) to ASCII using AVX2 vectorized comparisons.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn translate_indices_avx2(indices: __m256i, url_safe: bool) -> __m256i {
    let ge_26 = _mm256_cmpgt_epi8(indices, _mm256_set1_epi8(25));
    let ge_52 = _mm256_cmpgt_epi8(indices, _mm256_set1_epi8(51));
    let eq_62 = _mm256_cmpeq_epi8(indices, _mm256_set1_epi8(62));
    let eq_63 = _mm256_cmpeq_epi8(indices, _mm256_set1_epi8(63));

    let mut offset = _mm256_set1_epi8(65);
    offset = _mm256_add_epi8(offset, _mm256_and_si256(ge_26, _mm256_set1_epi8(6)));
    offset = _mm256_add_epi8(offset, _mm256_and_si256(ge_52, _mm256_set1_epi8(-75)));

    if !url_safe {
        offset = _mm256_add_epi8(offset, _mm256_and_si256(eq_62, _mm256_set1_epi8(-15)));
        offset = _mm256_add_epi8(offset, _mm256_and_si256(eq_63, _mm256_set1_epi8(-12)));
    } else {
        offset = _mm256_add_epi8(offset, _mm256_and_si256(eq_62, _mm256_set1_epi8(-13)));
        offset = _mm256_add_epi8(offset, _mm256_and_si256(eq_63, _mm256_set1_epi8(36)));
    }

    _mm256_add_epi8(indices, offset)
}


/// Helper to query CPUID in `#![no_std]` environments on x86_64.
#[cfg(all(target_arch = "x86_64", not(feature = "std")))]
fn cpuid_has_ssse3() -> bool {
    #[cfg(target_feature = "ssse3")]
    {
        true
    }
    #[cfg(not(target_feature = "ssse3"))]
    {
        let max_leaf = __cpuid(0).eax;
        if max_leaf < 1 {
            return false;
        }
        let leaf1 = __cpuid(1);
        // ECX bit 9 is SSSE3
        (leaf1.ecx & (1 << 9)) != 0
    }
}


/// Helper to query CPUID in `#![no_std]` environments on x86_64.
#[cfg(all(target_arch = "x86_64", not(feature = "std")))]
fn cpuid_has_avx2() -> bool {
    #[cfg(target_feature = "avx2")]
    {
        true
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        unsafe {
            let max_leaf = __cpuid(0).eax;
            if max_leaf < 7 {
                return false;
            }
            let leaf1 = __cpuid(1);
            // ECX bit 27 is OSXSAVE, bit 28 is AVX
            let osxsave = (leaf1.ecx & (1 << 27)) != 0;
            let avx = (leaf1.ecx & (1 << 28)) != 0;
            if !osxsave || !avx {
                return false;
            }
            let xcr0 = _xgetbv(0);
            // Bits 1 and 2 indicate OS has enabled XMM and YMM state saving
            if (xcr0 & 0x6) != 0x6 {
                return false;
            }
            let leaf7 = __cpuid_count(7, 0);
            // EBX bit 5 is AVX2
            (leaf7.ebx & (1 << 5)) != 0
        }
    }
}

/// Checks if the host x86_64 CPU supports AVX2.
#[inline(always)]
pub fn has_avx2() -> bool {
    #[cfg(target_feature = "avx2")]
    {
        true
    }
    #[cfg(all(target_arch = "x86_64", feature = "std", not(target_feature = "avx2")))]
    {
        std::is_x86_feature_detected!("avx2")
    }
    #[cfg(all(target_arch = "x86_64", not(feature = "std"), not(target_feature = "avx2")))]
    {
        cpuid_has_avx2()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// Checks if the host x86_64 CPU supports SSSE3.
#[inline(always)]
pub fn has_ssse3() -> bool {
    #[cfg(target_feature = "ssse3")]
    {
        true
    }
    #[cfg(all(target_arch = "x86_64", feature = "std", not(target_feature = "ssse3")))]
    {
        std::is_x86_feature_detected!("ssse3")
    }
    #[cfg(all(target_arch = "x86_64", not(feature = "std"), not(target_feature = "ssse3")))]
    {
        cpuid_has_ssse3()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// Helper to query CPUID in `#![no_std]` environments on x86_64 for AVX-512BW.
#[cfg(all(target_arch = "x86_64", not(feature = "std")))]
fn cpuid_has_avx512() -> bool {
    #[cfg(target_feature = "avx512bw")]
    {
        true
    }
    #[cfg(not(target_feature = "avx512bw"))]
    {
        unsafe {
            let max_leaf = __cpuid(0).eax;
            if max_leaf < 7 {
                return false;
            }
            let leaf1 = __cpuid(1);
            let osxsave = (leaf1.ecx & (1 << 27)) != 0;
            let avx = (leaf1.ecx & (1 << 28)) != 0;
            if !osxsave || !avx {
                return false;
            }
            let xcr0 = _xgetbv(0);
            // XCR0 bits 1, 2, 5, 6, 7 (XMM, YMM, opmask, ZMM_Hi256, Hi16_ZMM) -> 0xE6
            if (xcr0 & 0xE6) != 0xE6 {
                return false;
            }
            let leaf7 = __cpuid_count(7, 0);
            // EBX bit 16 is AVX512F, bit 30 is AVX512BW
            (leaf7.ebx & (1 << 16)) != 0 && (leaf7.ebx & (1 << 30)) != 0
        }
    }
}

/// Checks if the host x86_64 CPU supports AVX-512 (specifically AVX-512F and AVX-512BW).
#[inline(always)]
pub fn has_avx512() -> bool {
    #[cfg(target_feature = "avx512bw")]
    {
        true
    }
    #[cfg(all(target_arch = "x86_64", feature = "std", not(target_feature = "avx512bw")))]
    {
        std::is_x86_feature_detected!("avx512f") && std::is_x86_feature_detected!("avx512bw")
    }
    #[cfg(all(target_arch = "x86_64", not(feature = "std"), not(target_feature = "avx512bw")))]
    {
        cpuid_has_avx512()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// Checks if the host CPU supports WebAssembly 128-bit SIMD.
#[inline(always)]
pub fn has_wasm_simd128() -> bool {
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        true
    }
    #[cfg(not(all(target_arch = "wasm32", target_feature = "simd128")))]
    {
        false
    }
}

/// Translates 64 6-bit indices (0..63) to ASCII using AVX-512BW vectorized comparisons.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn translate_indices_avx512(indices: __m512i, url_safe: bool) -> __m512i {
    let ge_26 = _mm512_cmpgt_epi8_mask(indices, _mm512_set1_epi8(25));
    let ge_52 = _mm512_cmpgt_epi8_mask(indices, _mm512_set1_epi8(51));
    let eq_62 = _mm512_cmpeq_epi8_mask(indices, _mm512_set1_epi8(62));
    let eq_63 = _mm512_cmpeq_epi8_mask(indices, _mm512_set1_epi8(63));

    let mut offset = _mm512_set1_epi8(65);
    offset = _mm512_mask_add_epi8(offset, ge_26, offset, _mm512_set1_epi8(6));
    offset = _mm512_mask_add_epi8(offset, ge_52, offset, _mm512_set1_epi8(-75));

    if !url_safe {
        offset = _mm512_mask_add_epi8(offset, eq_62, offset, _mm512_set1_epi8(-15));
        offset = _mm512_mask_add_epi8(offset, eq_63, offset, _mm512_set1_epi8(-12));
    } else {
        offset = _mm512_mask_add_epi8(offset, eq_62, offset, _mm512_set1_epi8(-13));
        offset = _mm512_mask_add_epi8(offset, eq_63, offset, _mm512_set1_epi8(36));
    }

    _mm512_add_epi8(indices, offset)
}

/// Encodes 48 bytes to 64 Base64 ASCII bytes using AVX-512BW.
/// Returns (input_bytes_consumed, output_bytes_written).
///
/// # Safety
/// Caller must ensure the host CPU supports AVX-512BW, and `output` has at least 64 bytes of capacity.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
pub unsafe fn encode_avx512(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    static ENCODE_SHUFFLE_MASK: [i8; 64] = [
        2, 1, 0, -1, 5, 4, 3, -1, 8, 7, 6, -1, 11, 10, 9, -1,
        2, 1, 0, -1, 5, 4, 3, -1, 8, 7, 6, -1, 11, 10, 9, -1,
        2, 1, 0, -1, 5, 4, 3, -1, 8, 7, 6, -1, 11, 10, 9, -1,
        2, 1, 0, -1, 5, 4, 3, -1, 8, 7, 6, -1, 11, 10, 9, -1,
    ];

    unsafe {
        let shuffle_mask = _mm512_loadu_si512(ENCODE_SHUFFLE_MASK.as_ptr() as *const _);

        while in_idx + 48 <= in_len && out_idx + 64 <= out_len {
            let in0 = load_12_bytes(input.as_ptr().add(in_idx));
            let in1 = load_12_bytes(input.as_ptr().add(in_idx + 12));
            let in2 = load_12_bytes(input.as_ptr().add(in_idx + 24));
            let in3 = load_12_bytes(input.as_ptr().add(in_idx + 36));

            let in256_lo = _mm256_set_m128i(in1, in0);
            let in256_hi = _mm256_set_m128i(in3, in2);

            let in512 = _mm512_castsi256_si512(in256_lo);
            let in512 = _mm512_inserti64x4(in512, in256_hi, 1);

            let shuffled = _mm512_shuffle_epi8(in512, shuffle_mask);

            let t0 = _mm512_and_si512(shuffled, _mm512_set1_epi32(0x00FC0000));
            let t0 = _mm512_srli_epi32(t0, 18);

            let t1 = _mm512_and_si512(shuffled, _mm512_set1_epi32(0x0003F000));
            let t1 = _mm512_srli_epi32(t1, 12);

            let t2 = _mm512_and_si512(shuffled, _mm512_set1_epi32(0x00000FC0));
            let t2 = _mm512_srli_epi32(t2, 6);

            let t3 = _mm512_and_si512(shuffled, _mm512_set1_epi32(0x0000003F));

            let indices = _mm512_or_si512(
                _mm512_or_si512(t0, _mm512_slli_epi32(t1, 8)),
                _mm512_or_si512(_mm512_slli_epi32(t2, 16), _mm512_slli_epi32(t3, 24)),
            );

            let ascii = translate_indices_avx512(indices, url_safe);
            _mm512_storeu_si512(output.as_mut_ptr().add(out_idx) as *mut _, ascii);

            in_idx += 48;
            out_idx += 64;
        }
    }

    (in_idx, out_idx)
}

/// Decodes 64 Base64 ASCII bytes to 48 raw bytes using AVX-512BW.
/// Returns (input_bytes_consumed, output_bytes_written).
///
/// # Safety
/// Caller must ensure the host CPU supports AVX-512BW, and `output` has at least 48 bytes of capacity.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
pub unsafe fn decode_avx512(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    static DECODE_SHUFFLE_MASK: [i8; 64] = [
        2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
        2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
        2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
        2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1,
    ];

    unsafe {
        let decode_shuffle_mask = _mm512_loadu_si512(DECODE_SHUFFLE_MASK.as_ptr() as *const _);

        while in_idx + 64 <= in_len && out_idx + 48 <= out_len {
            let in_ptr = input.as_ptr().add(in_idx);
            let x = _mm512_loadu_si512(in_ptr as *const _);

            let ge_a = _mm512_cmpgt_epi8_mask(x, _mm512_set1_epi8(64));
            let le_z = _mm512_cmpgt_epi8_mask(_mm512_set1_epi8(91), x);
            let is_upper = ge_a & le_z;

            let ge_lower_a = _mm512_cmpgt_epi8_mask(x, _mm512_set1_epi8(96));
            let le_lower_z = _mm512_cmpgt_epi8_mask(_mm512_set1_epi8(123), x);
            let is_lower = ge_lower_a & le_lower_z;

            let ge_0 = _mm512_cmpgt_epi8_mask(x, _mm512_set1_epi8(47));
            let le_9 = _mm512_cmpgt_epi8_mask(_mm512_set1_epi8(58), x);
            let is_digit = ge_0 & le_9;

            let (is_valid_mask, offset) = if !url_safe {
                let is_plus = _mm512_cmpeq_epi8_mask(x, _mm512_set1_epi8(43));
                let is_slash = _mm512_cmpeq_epi8_mask(x, _mm512_set1_epi8(47));
                let valid = is_upper | is_lower | is_digit | is_plus | is_slash;

                let mut off = _mm512_setzero_si512();
                off = _mm512_mask_mov_epi8(off, is_upper, _mm512_set1_epi8(-65));
                off = _mm512_mask_mov_epi8(off, is_lower, _mm512_set1_epi8(-71));
                off = _mm512_mask_mov_epi8(off, is_digit, _mm512_set1_epi8(4));
                off = _mm512_mask_mov_epi8(off, is_plus, _mm512_set1_epi8(19));
                off = _mm512_mask_mov_epi8(off, is_slash, _mm512_set1_epi8(16));

                (valid, off)
            } else {
                let is_dash = _mm512_cmpeq_epi8_mask(x, _mm512_set1_epi8(45));
                let is_under = _mm512_cmpeq_epi8_mask(x, _mm512_set1_epi8(95));
                let valid = is_upper | is_lower | is_digit | is_dash | is_under;

                let mut off = _mm512_setzero_si512();
                off = _mm512_mask_mov_epi8(off, is_upper, _mm512_set1_epi8(-65));
                off = _mm512_mask_mov_epi8(off, is_lower, _mm512_set1_epi8(-71));
                off = _mm512_mask_mov_epi8(off, is_digit, _mm512_set1_epi8(4));
                off = _mm512_mask_mov_epi8(off, is_dash, _mm512_set1_epi8(17));
                off = _mm512_mask_mov_epi8(off, is_under, _mm512_set1_epi8(-32));

                (valid, off)
            };

            // If any of the 64 characters is not valid Base64 symbol (or is '='), fallback to scalar
            if is_valid_mask != !0u64 {
                break;
            }

            let indices = _mm512_add_epi8(x, offset);

            let merge_pairs = _mm512_maddubs_epi16(indices, _mm512_set1_epi16(0x0140));
            let merge_dwords = _mm512_madd_epi16(merge_pairs, _mm512_set1_epi32(0x00011000));
            let shuffled = _mm512_shuffle_epi8(merge_dwords, decode_shuffle_mask);

            let shuffled_lo = _mm512_castsi512_si256(shuffled);
            let shuffled_hi = _mm512_extracti64x4_epi64(shuffled, 1);

            let res0 = _mm256_castsi256_si128(shuffled_lo);
            let res1 = _mm256_extracti128_si256(shuffled_lo, 1);
            let res2 = _mm256_castsi256_si128(shuffled_hi);
            let res3 = _mm256_extracti128_si256(shuffled_hi, 1);

            let out_ptr = output.as_mut_ptr().add(out_idx);

            // Lane 0: 12 bytes
            core::ptr::write_unaligned(out_ptr as *mut u64, _mm_cvtsi128_si64(res0) as u64);
            core::ptr::write_unaligned(out_ptr.add(8) as *mut u32, _mm_cvtsi128_si32(_mm_srli_si128(res0, 8)) as u32);

            // Lane 1: 12 bytes
            core::ptr::write_unaligned(out_ptr.add(12) as *mut u64, _mm_cvtsi128_si64(res1) as u64);
            core::ptr::write_unaligned(out_ptr.add(20) as *mut u32, _mm_cvtsi128_si32(_mm_srli_si128(res1, 8)) as u32);

            // Lane 2: 12 bytes
            core::ptr::write_unaligned(out_ptr.add(24) as *mut u64, _mm_cvtsi128_si64(res2) as u64);
            core::ptr::write_unaligned(out_ptr.add(32) as *mut u32, _mm_cvtsi128_si32(_mm_srli_si128(res2, 8)) as u32);

            // Lane 3: 12 bytes
            core::ptr::write_unaligned(out_ptr.add(36) as *mut u64, _mm_cvtsi128_si64(res3) as u64);
            core::ptr::write_unaligned(out_ptr.add(44) as *mut u32, _mm_cvtsi128_si32(_mm_srli_si128(res3, 8)) as u32);

            in_idx += 64;
            out_idx += 48;
        }
    }

    (in_idx, out_idx)
}

/// Encodes 12 bytes to 16 Base64 ASCII bytes using SSSE3 (128-bit SIMD).
///
/// # Safety
/// Caller must ensure the host CPU supports SSSE3, and `output` has at least 16 bytes of capacity.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3")]
pub unsafe fn encode_ssse3(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    unsafe {
        let shuffle_mask = _mm_setr_epi8(2, 1, 0, -1, 5, 4, 3, -1, 8, 7, 6, -1, 11, 10, 9, -1);

        while in_idx + 12 <= in_len && out_idx + 16 <= out_len {
            let in12 = load_12_bytes(input.as_ptr().add(in_idx));
            let shuffled = _mm_shuffle_epi8(in12, shuffle_mask);

            let t0 = _mm_and_si128(shuffled, _mm_set1_epi32(0x00FC0000));
            let t0 = _mm_srli_epi32(t0, 18);

            let t1 = _mm_and_si128(shuffled, _mm_set1_epi32(0x0003F000));
            let t1 = _mm_srli_epi32(t1, 12);

            let t2 = _mm_and_si128(shuffled, _mm_set1_epi32(0x00000FC0));
            let t2 = _mm_srli_epi32(t2, 6);

            let t3 = _mm_and_si128(shuffled, _mm_set1_epi32(0x0000003F));

            let indices = _mm_or_si128(
                _mm_or_si128(t0, _mm_slli_epi32(t1, 8)),
                _mm_or_si128(_mm_slli_epi32(t2, 16), _mm_slli_epi32(t3, 24)),
            );

            let ascii = translate_indices_ssse3(indices, url_safe);
            _mm_storeu_si128(output.as_mut_ptr().add(out_idx) as *mut __m128i, ascii);

            in_idx += 12;
            out_idx += 16;
        }
    }

    (in_idx, out_idx)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3")]
unsafe fn translate_indices_ssse3(indices: __m128i, url_safe: bool) -> __m128i {
    let ge_26 = _mm_cmpgt_epi8(indices, _mm_set1_epi8(25));
    let ge_52 = _mm_cmpgt_epi8(indices, _mm_set1_epi8(51));
    let eq_62 = _mm_cmpeq_epi8(indices, _mm_set1_epi8(62));
    let eq_63 = _mm_cmpeq_epi8(indices, _mm_set1_epi8(63));

    let mut offset = _mm_set1_epi8(65);
    offset = _mm_add_epi8(offset, _mm_and_si128(ge_26, _mm_set1_epi8(6)));
    offset = _mm_add_epi8(offset, _mm_and_si128(ge_52, _mm_set1_epi8(-75)));

    if !url_safe {
        offset = _mm_add_epi8(offset, _mm_and_si128(eq_62, _mm_set1_epi8(-15)));
        offset = _mm_add_epi8(offset, _mm_and_si128(eq_63, _mm_set1_epi8(-12)));
    } else {
        offset = _mm_add_epi8(offset, _mm_and_si128(eq_62, _mm_set1_epi8(-13)));
        offset = _mm_add_epi8(offset, _mm_and_si128(eq_63, _mm_set1_epi8(36)));
    }

    _mm_add_epi8(indices, offset)
}

/// Decodes 16 Base64 ASCII bytes to 12 raw bytes using SSSE3 (128-bit SIMD).
///
/// # Safety
/// Caller must ensure the host CPU supports SSSE3, and `output` has at least 12 bytes of capacity.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3")]
pub unsafe fn decode_ssse3(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    unsafe {
        let decode_shuffle_mask =
            _mm_setr_epi8(2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1);

        while in_idx + 16 <= in_len && out_idx + 12 <= out_len {
            let in_ptr = input.as_ptr().add(in_idx);
            let x = _mm_loadu_si128(in_ptr as *const __m128i);

            let ge_a = _mm_cmpgt_epi8(x, _mm_set1_epi8(64));
            let le_z = _mm_cmpgt_epi8(_mm_set1_epi8(91), x);
            let is_upper = _mm_and_si128(ge_a, le_z);

            let ge_lower_a = _mm_cmpgt_epi8(x, _mm_set1_epi8(96));
            let le_lower_z = _mm_cmpgt_epi8(_mm_set1_epi8(123), x);
            let is_lower = _mm_and_si128(ge_lower_a, le_lower_z);

            let ge_0 = _mm_cmpgt_epi8(x, _mm_set1_epi8(47));
            let le_9 = _mm_cmpgt_epi8(_mm_set1_epi8(58), x);
            let is_digit = _mm_and_si128(ge_0, le_9);

            let (is_valid, offset) = if !url_safe {
                let is_plus = _mm_cmpeq_epi8(x, _mm_set1_epi8(43));
                let is_slash = _mm_cmpeq_epi8(x, _mm_set1_epi8(47));
                let valid = _mm_or_si128(
                    _mm_or_si128(is_upper, is_lower),
                    _mm_or_si128(is_digit, _mm_or_si128(is_plus, is_slash)),
                );

                let mut off = _mm_setzero_si128();
                off = _mm_or_si128(off, _mm_and_si128(is_upper, _mm_set1_epi8(-65)));
                off = _mm_or_si128(off, _mm_and_si128(is_lower, _mm_set1_epi8(-71)));
                off = _mm_or_si128(off, _mm_and_si128(is_digit, _mm_set1_epi8(4)));
                off = _mm_or_si128(off, _mm_and_si128(is_plus, _mm_set1_epi8(19)));
                off = _mm_or_si128(off, _mm_and_si128(is_slash, _mm_set1_epi8(16)));

                (valid, off)
            } else {
                let is_dash = _mm_cmpeq_epi8(x, _mm_set1_epi8(45));
                let is_under = _mm_cmpeq_epi8(x, _mm_set1_epi8(95));
                let valid = _mm_or_si128(
                    _mm_or_si128(is_upper, is_lower),
                    _mm_or_si128(is_digit, _mm_or_si128(is_dash, is_under)),
                );

                let mut off = _mm_setzero_si128();
                off = _mm_or_si128(off, _mm_and_si128(is_upper, _mm_set1_epi8(-65)));
                off = _mm_or_si128(off, _mm_and_si128(is_lower, _mm_set1_epi8(-71)));
                off = _mm_or_si128(off, _mm_and_si128(is_digit, _mm_set1_epi8(4)));
                off = _mm_or_si128(off, _mm_and_si128(is_dash, _mm_set1_epi8(17)));
                off = _mm_or_si128(off, _mm_and_si128(is_under, _mm_set1_epi8(-32)));

                (valid, off)
            };

            let valid_mask = _mm_movemask_epi8(is_valid);
            if (valid_mask as u32 & 0xFFFF) != 0xFFFF {
                break;
            }

            let indices = _mm_add_epi8(x, offset);

            let merge_pairs = _mm_maddubs_epi16(indices, _mm_set1_epi16(0x0140));
            let merge_dwords = _mm_madd_epi16(merge_pairs, _mm_set1_epi32(0x00011000));
            let shuffled = _mm_shuffle_epi8(merge_dwords, decode_shuffle_mask);

            let out_ptr = output.as_mut_ptr().add(out_idx);
            core::ptr::write_unaligned(out_ptr as *mut u64, _mm_cvtsi128_si64(shuffled) as u64);
            core::ptr::write_unaligned(
                out_ptr.add(8) as *mut u32,
                _mm_cvtsi128_si32(_mm_srli_si128(shuffled, 8)) as u32,
            );

            in_idx += 16;
            out_idx += 12;
        }
    }

    (in_idx, out_idx)
}

/// Decodes 32 Base64 ASCII bytes to 24 raw bytes using AVX2.
/// Returns (input_bytes_consumed, output_bytes_written).
/// Breaks out if padding ('=') or invalid characters are encountered.
///
/// # Safety
/// Caller must ensure the host CPU supports AVX2, and `output` has at least 24 bytes of capacity.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn decode_avx2(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    unsafe {
        let decode_shuffle_mask = _mm256_setr_epi8(
            2, 1, 0, 6, 5, 4, 10, 9, 8, 14, 13, 12, -1, -1, -1, -1, 2, 1, 0, 6, 5, 4, 10, 9, 8, 14,
            13, 12, -1, -1, -1, -1,
        );

        while in_idx + 32 <= in_len && out_idx + 24 <= out_len {
            let in_ptr = input.as_ptr().add(in_idx);
            let x = _mm256_loadu_si256(in_ptr as *const __m256i);

            let ge_a = _mm256_cmpgt_epi8(x, _mm256_set1_epi8(64));
            let le_z = _mm256_cmpgt_epi8(_mm256_set1_epi8(91), x);
            let is_upper = _mm256_and_si256(ge_a, le_z);

            let ge_lower_a = _mm256_cmpgt_epi8(x, _mm256_set1_epi8(96));
            let le_lower_z = _mm256_cmpgt_epi8(_mm256_set1_epi8(123), x);
            let is_lower = _mm256_and_si256(ge_lower_a, le_lower_z);

            let ge_0 = _mm256_cmpgt_epi8(x, _mm256_set1_epi8(47));
            let le_9 = _mm256_cmpgt_epi8(_mm256_set1_epi8(58), x);
            let is_digit = _mm256_and_si256(ge_0, le_9);

            let (is_valid, offset) = if !url_safe {
                let is_plus = _mm256_cmpeq_epi8(x, _mm256_set1_epi8(43));
                let is_slash = _mm256_cmpeq_epi8(x, _mm256_set1_epi8(47));
                let valid = _mm256_or_si256(
                    _mm256_or_si256(is_upper, is_lower),
                    _mm256_or_si256(is_digit, _mm256_or_si256(is_plus, is_slash)),
                );

                let mut off = _mm256_setzero_si256();
                off = _mm256_or_si256(off, _mm256_and_si256(is_upper, _mm256_set1_epi8(-65)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_lower, _mm256_set1_epi8(-71)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_digit, _mm256_set1_epi8(4)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_plus, _mm256_set1_epi8(19)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_slash, _mm256_set1_epi8(16)));

                (valid, off)
            } else {
                let is_dash = _mm256_cmpeq_epi8(x, _mm256_set1_epi8(45));
                let is_under = _mm256_cmpeq_epi8(x, _mm256_set1_epi8(95));
                let valid = _mm256_or_si256(
                    _mm256_or_si256(is_upper, is_lower),
                    _mm256_or_si256(is_digit, _mm256_or_si256(is_dash, is_under)),
                );

                let mut off = _mm256_setzero_si256();
                off = _mm256_or_si256(off, _mm256_and_si256(is_upper, _mm256_set1_epi8(-65)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_lower, _mm256_set1_epi8(-71)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_digit, _mm256_set1_epi8(4)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_dash, _mm256_set1_epi8(17)));
                off = _mm256_or_si256(off, _mm256_and_si256(is_under, _mm256_set1_epi8(-32)));

                (valid, off)
            };

            // If any character is not valid Base64 symbol (or is '='), fallback to scalar
            let valid_mask = _mm256_movemask_epi8(is_valid);
            if valid_mask != -1 {
                break;
            }

            let indices = _mm256_add_epi8(x, offset);

            // Pack 6-bit sextets into 8-bit bytes
            let merge_pairs = _mm256_maddubs_epi16(indices, _mm256_set1_epi16(0x0140));
            let merge_dwords = _mm256_madd_epi16(merge_pairs, _mm256_set1_epi32(0x00011000));
            let shuffled = _mm256_shuffle_epi8(merge_dwords, decode_shuffle_mask);

            let out_ptr = output.as_mut_ptr().add(out_idx);
            let res_lo = _mm256_castsi256_si128(shuffled);
            let res_hi = _mm256_extracti128_si256(shuffled, 1);

            // Lane 0: 12 bytes
            core::ptr::write_unaligned(out_ptr as *mut u64, _mm_cvtsi128_si64(res_lo) as u64);
            core::ptr::write_unaligned(
                out_ptr.add(8) as *mut u32,
                _mm_cvtsi128_si32(_mm_srli_si128(res_lo, 8)) as u32,
            );

            // Lane 1: 12 bytes
            core::ptr::write_unaligned(
                out_ptr.add(12) as *mut u64,
                _mm_cvtsi128_si64(res_hi) as u64,
            );
            core::ptr::write_unaligned(
                out_ptr.add(20) as *mut u32,
                _mm_cvtsi128_si32(_mm_srli_si128(res_hi, 8)) as u32,
            );

            in_idx += 32;
            out_idx += 24;
        }
    }

    (in_idx, out_idx)
}

/// Checks if the host CPU supports ARM NEON (always true on AArch64).
#[inline(always)]
pub fn has_neon() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        true
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        false
    }
}

/// Encodes 12 bytes to 16 Base64 ASCII bytes on AArch64.
///
/// # Safety
/// Caller must ensure `input` is valid for reads and `output` has sufficient capacity.
#[cfg(target_arch = "aarch64")]
pub unsafe fn encode_neon(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    let enc_table = if !url_safe {
        crate::alphabet::STANDARD_ENCODE_TABLE
    } else {
        crate::alphabet::URL_SAFE_ENCODE_TABLE
    };

    while in_idx + 12 <= in_len && out_idx + 16 <= out_len {
        for chunk in 0..4 {
            let offset = in_idx + chunk * 3;
            let b0 = *input.get_unchecked(offset) as u32;
            let b1 = *input.get_unchecked(offset + 1) as u32;
            let b2 = *input.get_unchecked(offset + 2) as u32;

            let n = (b0 << 16) | (b1 << 8) | b2;
            let c0 = *enc_table.get_unchecked((n >> 18) as usize);
            let c1 = *enc_table.get_unchecked(((n >> 12) & 0x3F) as usize);
            let c2 = *enc_table.get_unchecked(((n >> 6) & 0x3F) as usize);
            let c3 = *enc_table.get_unchecked((n & 0x3F) as usize);

            let word = u32::from_ne_bytes([c0, c1, c2, c3]);
            core::ptr::write_unaligned(
                output.as_mut_ptr().add(out_idx + chunk * 4) as *mut u32,
                word,
            );
        }
        in_idx += 12;
        out_idx += 16;
    }

    (in_idx, out_idx)
}

/// Decodes 16 Base64 ASCII bytes to 12 raw bytes on AArch64.
///
/// # Safety
/// Caller must ensure `input` is valid for reads and `output` has sufficient capacity.
#[cfg(target_arch = "aarch64")]
pub unsafe fn decode_neon(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    let dec_table = if !url_safe {
        &crate::alphabet::STANDARD_DECODE_TABLE
    } else {
        &crate::alphabet::URL_SAFE_NO_PAD_DECODE_TABLE
    };

    while in_idx + 16 <= in_len && out_idx + 12 <= out_len {
        let mut combined = 0u8;
        let mut sextets = [0u8; 16];

        for i in 0..16 {
            let c = *input.get_unchecked(in_idx + i);
            let s = *dec_table.get_unchecked(c as usize);
            sextets[i] = s;
            combined |= s;
        }

        if (combined & 0xC0) != 0 {
            break;
        }

        for chunk in 0..4 {
            let base = chunk * 4;
            let s0 = sextets[base] as u32;
            let s1 = sextets[base + 1] as u32;
            let s2 = sextets[base + 2] as u32;
            let s3 = sextets[base + 3] as u32;
            let n = (s0 << 18) | (s1 << 12) | (s2 << 6) | s3;

            let out_offset = out_idx + chunk * 3;
            *output.get_unchecked_mut(out_offset) = (n >> 16) as u8;
            *output.get_unchecked_mut(out_offset + 1) = (n >> 8) as u8;
            *output.get_unchecked_mut(out_offset + 2) = n as u8;
        }

        in_idx += 16;
        out_idx += 12;
    }

    (in_idx, out_idx)
}

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32::*;

#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
unsafe fn translate_indices_wasm(indices: v128, url_safe: bool) -> v128 {
    let ge_26 = i8x16_gt(indices, i8x16_splat(25));
    let ge_52 = i8x16_gt(indices, i8x16_splat(51));
    let eq_62 = i8x16_eq(indices, i8x16_splat(62));
    let eq_63 = i8x16_eq(indices, i8x16_splat(63));

    let mut offset = i8x16_splat(65);
    offset = i8x16_add(offset, v128_and(ge_26, i8x16_splat(6)));
    offset = i8x16_add(offset, v128_and(ge_52, i8x16_splat(-75)));

    if !url_safe {
        offset = i8x16_add(offset, v128_and(eq_62, i8x16_splat(-15)));
        offset = i8x16_add(offset, v128_and(eq_63, i8x16_splat(-12)));
    } else {
        offset = i8x16_add(offset, v128_and(eq_62, i8x16_splat(-13)));
        offset = i8x16_add(offset, v128_and(eq_63, i8x16_splat(36)));
    }

    i8x16_add(indices, offset)
}

/// Encodes 12 bytes to 16 Base64 ASCII bytes on WebAssembly using 128-bit SIMD.
///
/// # Safety
/// Caller must ensure target supports WASM SIMD128 and output has at least 16 bytes.
#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
pub unsafe fn encode_wasm_simd(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    unsafe {
        while in_idx + 12 <= in_len && out_idx + 16 <= out_len {
            let b64 = core::ptr::read_unaligned(input.as_ptr().add(in_idx) as *const u64);
            let b32 = core::ptr::read_unaligned(input.as_ptr().add(in_idx + 8) as *const u32);
            let in128 = u64x2(b64, b32 as u64);

            let shuffled = i8x16_shuffle::<2, 1, 0, 255, 5, 4, 3, 255, 8, 7, 6, 255, 11, 10, 9, 255>(in128, in128);

            let t0 = u32x4_shr(v128_and(shuffled, u32x4_splat(0x00FC0000)), 18);
            let t1 = u32x4_shr(v128_and(shuffled, u32x4_splat(0x0003F000)), 12);
            let t2 = u32x4_shr(v128_and(shuffled, u32x4_splat(0x00000FC0)), 6);
            let t3 = v128_and(shuffled, u32x4_splat(0x0000003F));

            let indices = v128_or(
                v128_or(t0, u32x4_shl(t1, 8)),
                v128_or(u32x4_shl(t2, 16), u32x4_shl(t3, 24)),
            );

            let ascii = translate_indices_wasm(indices, url_safe);
            v128_store(output.as_mut_ptr().add(out_idx) as *mut v128, ascii);

            in_idx += 12;
            out_idx += 16;
        }
    }

    (in_idx, out_idx)
}

/// Decodes 16 Base64 ASCII bytes to 12 raw bytes on WebAssembly using 128-bit SIMD.
///
/// # Safety
/// Caller must ensure target supports WASM SIMD128 and output has at least 12 bytes.
#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
pub unsafe fn decode_wasm_simd(input: &[u8], output: &mut [u8], url_safe: bool) -> (usize, usize) {
    let mut in_idx = 0;
    let mut out_idx = 0;
    let in_len = input.len();
    let out_len = output.len();

    unsafe {
        while in_idx + 16 <= in_len && out_idx + 12 <= out_len {
            let x = v128_load(input.as_ptr().add(in_idx) as *const v128);

            let ge_a = i8x16_gt(x, i8x16_splat(64));
            let le_z = i8x16_gt(i8x16_splat(91), x);
            let is_upper = v128_and(ge_a, le_z);

            let ge_lower_a = i8x16_gt(x, i8x16_splat(96));
            let le_lower_z = i8x16_gt(i8x16_splat(123), x);
            let is_lower = v128_and(ge_lower_a, le_lower_z);

            let ge_0 = i8x16_gt(x, i8x16_splat(47));
            let le_9 = i8x16_gt(i8x16_splat(58), x);
            let is_digit = v128_and(ge_0, le_9);

            let (is_valid, offset) = if !url_safe {
                let is_plus = i8x16_eq(x, i8x16_splat(43));
                let is_slash = i8x16_eq(x, i8x16_splat(47));
                let valid = v128_or(
                    v128_or(is_upper, is_lower),
                    v128_or(is_digit, v128_or(is_plus, is_slash)),
                );

                let mut off = i8x16_splat(0);
                off = v128_or(off, v128_and(is_upper, i8x16_splat(-65)));
                off = v128_or(off, v128_and(is_lower, i8x16_splat(-71)));
                off = v128_or(off, v128_and(is_digit, i8x16_splat(4)));
                off = v128_or(off, v128_and(is_plus, i8x16_splat(19)));
                off = v128_or(off, v128_and(is_slash, i8x16_splat(16)));

                (valid, off)
            } else {
                let is_dash = i8x16_eq(x, i8x16_splat(45));
                let is_under = i8x16_eq(x, i8x16_splat(95));
                let valid = v128_or(
                    v128_or(is_upper, is_lower),
                    v128_or(is_digit, v128_or(is_dash, is_under)),
                );

                let mut off = i8x16_splat(0);
                off = v128_or(off, v128_and(is_upper, i8x16_splat(-65)));
                off = v128_or(off, v128_and(is_lower, i8x16_splat(-71)));
                off = v128_or(off, v128_and(is_digit, i8x16_splat(4)));
                off = v128_or(off, v128_and(is_dash, i8x16_splat(17)));
                off = v128_or(off, v128_and(is_under, i8x16_splat(-32)));

                (valid, off)
            };

            if i8x16_bitmask(is_valid) != 0xFFFF {
                break;
            }

            let indices = i8x16_add(x, offset);

            let mut sextets = [0u8; 16];
            v128_store(sextets.as_mut_ptr() as *mut v128, indices);

            for chunk in 0..4 {
                let base = chunk * 4;
                let s0 = sextets[base] as u32;
                let s1 = sextets[base + 1] as u32;
                let s2 = sextets[base + 2] as u32;
                let s3 = sextets[base + 3] as u32;
                let n = (s0 << 18) | (s1 << 12) | (s2 << 6) | s3;

                let out_offset = out_idx + chunk * 3;
                *output.get_unchecked_mut(out_offset) = (n >> 16) as u8;
                *output.get_unchecked_mut(out_offset + 1) = (n >> 8) as u8;
                *output.get_unchecked_mut(out_offset + 2) = n as u8;
            }

            in_idx += 16;
            out_idx += 12;
        }
    }

    (in_idx, out_idx)
}
