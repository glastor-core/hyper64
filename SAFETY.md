# hyper64 Safety & Mathematical Invariants

This document formally records the memory safety, pointer invariants, and verification guarantees of all `unsafe` code blocks within `hyper64`.

---

## 1. Safety Architecture Overview

`hyper64` is designed under the **"zero runtime overhead, absolute memory integrity"** paradigm. While the public surface is 100% safe Rust, high-throughput SIMD vectorization and zero-allocation in-place decoding utilize targeted `unsafe` blocks. Every `unsafe` block satisfies strict formal invariants detailed below.

---

## 2. In-Place Decoding Invariant ($r_k \ge w_k$)

### Theorem
For any Base64 encoded byte slice $B$ of length $L \ge 0$, in-place decoding into $B$ will **never overwrite unread input data**.

### Mathematical Proof
Let $r_k$ denote the read pointer index and $w_k$ denote the write pointer index at discrete iteration step $k$:
- Base state: $r_0 = 0$, $w_0 = 0 \implies r_0 \ge w_0$.
- **AVX2 Vectorized Step**: Consumes $\Delta r = 32$ ASCII bytes, writes $\Delta w = 24$ raw decoded bytes:
  $$\Delta r - \Delta w = 32 - 24 = 8 > 0$$
  $$r_{k+1} - w_{k+1} = (r_k + 32) - (w_k + 24) = (r_k - w_k) + 8 \ge r_k - w_k$$
- **SSSE3 / NEON Vectorized Step**: Consumes $\Delta r = 16$ ASCII bytes, writes $\Delta w = 12$ raw decoded bytes:
  $$\Delta r - \Delta w = 16 - 12 = 4 > 0$$
- **SWAR Permissive Step**: Consumes $\Delta r = 8$ ASCII bytes, writes $\Delta w = 6$ raw decoded bytes:
  $$\Delta r - \Delta w = 8 - 6 = 2 > 0$$
- **Scalar Quad Step**: Consumes $\Delta r = 4$ ASCII bytes, writes $\Delta w \in \{1, 2, 3\}$ raw bytes:
  $$\Delta r - \Delta w \ge 4 - 3 = 1 > 0$$
- **Whitespace / Padding Skip**: Consumes $\Delta r \ge 1$ byte without writing ($\Delta w = 0$):
  $$r_{k+1} - w_{k+1} = r_k + \Delta r - w_k > r_k - w_k$$

**Conclusion:** At every transition step $k$, $r_k - w_k$ is monotonically non-decreasing. Because $r_k \ge w_k$ for all $k$, the write pointer is strictly behind the read pointer. Input data is consumed before the underlying memory location can be overwritten.

---

## 3. Pointer Alignment and Unaligned Loads/Stores

- **x86_64 and AArch64 Architectural Compliance**: Modern x86_64 and ARMv8-A architectures support unaligned memory accesses natively without faulting.
- **Explicit Unaligned Primitives**: `hyper64` never casts unaligned pointers to aligned reference types (`&u32` or `&u64`), which would violate Rust strict aliasing rules. Instead, it exclusively uses:
  - `core::ptr::read_unaligned` / `core::ptr::write_unaligned`
  - `_mm256_loadu_si256` / `_mm256_storeu_si256`
  - `_mm_loadu_si128` / `_mm_storeu_si128`
- **Bounds Invariant**: Whenever `read_unaligned::<u64>(ptr.add(r))` or `_mm256_loadu_si256` is invoked, the loop guard unconditionally guarantees $r + N \le L$ where $N$ is the chunk byte size. No out-of-bounds page faults can occur.

---

## 4. Hardware Feature Detection and CPUID Invariants

- Functions decorated with `#[target_feature(enable = "avx2")]` or `#[target_feature(enable = "ssse3")]` must only execute on CPUs that support those instruction sets.
- `hyper64` enforces dynamic hardware gating:
  1. **Compile-Time**: If the binary is built with `-C target-feature=+avx2` or `-C target-cpu=native`, the check is evaluated as `true` at compile time with zero branch overhead.
  2. **Standard Library (`feature = "std"`)**: Uses `std::is_x86_feature_detected!("avx2")` backed by OS kernel CPUID caches.
  3. **Embedded / Kernel (`#![no_std]`)**: Directly queries `core::arch::x86_64::__cpuid` and validates `_xgetbv(0)` to confirm that the operating system has actively enabled YMM/XMM register saving in context switches (`XCR0[2:1] == 11b`).

---

## 5. Constant-Time Timing Invariant

- Cryptographic functions `encode_slice_ct` and `decode_slice_ct` must not leak secret length or value through timing or data-dependent cache access patterns.
- **Lookup Elimination**: Replaces all table lookups with branchless arithmetic and bitwise logic (`&`, `|`, `^`, `>>`, `<<`).
- **Optimization Fence**: Invokes `core::hint::black_box` on temporary intermediate accumulators, preventing the LLVM compiler from reintroducing conditional jumps (`cmov`, branches) during dead-code elimination or vectorization.

---

## 6. Zero Double-Write Memory Initialization

- High-throughput heap allocators (`encode`, `decode`) avoid double memory writes:
  1. `Vec::with_capacity(cap)` allocates uninitialized heap memory.
  2. A temporary mutable pointer `vec.as_mut_ptr()` is passed to the encoder/decoder.
  3. `decode_to_slice` or `encode_to_slice` writes exactly $M \le \text{cap}$ bytes into the buffer.
  4. `vec.set_len(M)` is called **strictly after** the bytes have been written.
  5. No uninitialized memory is ever read or exposed across safe boundaries.
- **UTF-8 Safety**: Base64 output consists entirely of ASCII characters (byte values $0 \le c \le 127$), which are structurally guaranteed to be valid UTF-8. Calling `String::from_utf8_unchecked(vec)` is mathematically sound.

---

## 7. Fuzzing and Differential Testing Verification

The safety invariants above are continuously verified by:
- **Differential Oracle Test**: 2,048 randomized iterations comparing `hyper64` against `crates.io/crates/base64` 0.22.
- **Property-Based In-Place Mutation Test**: Validates zero byte corruption across random payload distributions.
- **libFuzzer Targets**: `fuzz/fuzz_targets/{fuzz_decode, fuzz_roundtrip, fuzz_pem}` running under ASan (AddressSanitizer) and UBSan (UndefinedBehaviorSanitizer).

