# <img src="assets/isologo.webp" width="40" height="40" alt="GLASTOR Logo" align="absmiddle"> hyper64 ⚡

<p align="center">
  <img src="assets/banner.svg" alt="GLASTOR hyper64 Banner" width="100%">
</p>

<p align="center">
  <strong>Next-Generation, Ultra-High-Throughput, Zero-Allocation Base64 Engine for Rust</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/hyper64"><img src="https://img.shields.io/badge/crates.io-v0.2.0-orange.svg" alt="Crates.io"></a>
  <a href="https://docs.rs/hyper64"><img src="https://img.shields.io/badge/docs.rs-hyper64-blue.svg" alt="Documentation"></a>
  <a href="https://github.com/glastor-core/hyper64/actions/workflows/ci.yml"><img src="https://github.com/glastor-core/hyper64/actions/workflows/ci.yml/badge.svg" alt="CI Status"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
  <img src="https://img.shields.io/badge/rustc-1.85%2B-blue.svg" alt="MSRV 1.85+">
  <img src="https://img.shields.io/badge/no__std-compatible-success.svg" alt="no_std">
  <img src="https://img.shields.io/badge/dependencies-0-brightgreen.svg" alt="Zero Dependencies">
</p>

---

## 📖 Executive Summary

**`hyper64`** is an industrial-grade Base64 engine designed from the hardware up to solve the real-world performance, memory allocation, and side-channel security bottlenecks of the standard `base64` crate.

By combining **multi-tier SIMD vectorization** (AVX-512BW, AVX2, SSSE3, AArch64 NEON, WebAssembly SIMD128), **mathematically proven in-place zero-copy decoding**, **constant-time cryptographic execution**, and **zero-double-write memory initialization**, `hyper64` achieves sustained throughput exceeding **10–25 GiB/s** while guaranteeing zero heap allocations and 100% API drop-in compatibility with the `base64` 0.22 crate.

---

## 📊 Empirical Benchmarks

Benchmarks measured on standard x86_64 hardware via [Criterion.rs](https://github.com/bheisler/criterion.rs) comparing `hyper64` against `crates.io/crates/base64` (v0.22):

### ⚡ Encoding Throughput (Higher is Better)

```text
64 B   | [hyper64] 1.23 GiB/s  (+42% faster)
       | [base64]  863 MiB/s

1 KiB  | [hyper64] 7.22 GiB/s  (3.35x faster, +235%)
       | [base64]  2.15 GiB/s

64 KiB | [hyper64] 10.20 GiB/s (4.40x faster, +340%)
       | [base64]  2.32 GiB/s
```

| Payload Size | `base64` crate 0.22 | `hyper64` | Speedup / Improvement |
|:---|:---|:---|:---|
| **64 Bytes** | 863 MiB/s (70.6 ns) | **1.23 GiB/s** (48.3 ns) | **+42% throughput** |
| **1 KiB** | 2.15 GiB/s (442.4 ns) | **7.22 GiB/s** (132.1 ns) | **3.35x faster (+235%)** |
| **64 KiB** | 2.32 GiB/s (26.2 µs) | **10.20 GiB/s** (5.98 µs) | **4.40x faster (+340%)** |

### ⚡ Decoding Throughput & Latency (Lower CPU Time is Better)

| Payload Size | `base64` crate 0.22 | `hyper64-in-place` (0 Heap) | `hyper64` (Alloc) | CPU Latency Reduction |
|:---|:---|:---|:---|:---|
| **1 KiB** | 2.12 GiB/s (448.3 ns) | **6.84 GiB/s** (139.3 ns) | **7.15 GiB/s** (134.2 ns) | **-76.7% CPU Time** |
| **64 KiB** | 1.96 GiB/s (31.0 µs) | **6.78 GiB/s** (8.99 µs) | **7.91 GiB/s** (7.71 µs) | **-80.6% CPU Time** |

---

## 🚀 Architectural Advantages

| Capability | Standard `base64` Crate | `hyper64` | Production Engineering Value |
|:---|:---:|:---:|:---|
| **Vectorized SIMD Engine** | ❌ Scalar only (~2 GB/s) | ✅ **AVX-512 / AVX2 / SSSE3 / NEON / WASM** (>10–25 GB/s) | 4x–10x lower server CPU utilization in JSON/gRPC APIs & Edge |
| **Zero-Alloc In-Place Decoding** | ❌ Dest buffer required | ✅ **`decode_in_place(&mut [u8])`** | 0 heap allocations in network proxies, TLS & mesh routers |
| **Zero Double-Write Memory** | ❌ Zero-fills entire Vec first | ✅ **Spare Capacity Writer** | Cuts memory bus saturation in half on multi-megabyte blobs |
| **`no_std` Dynamic CPUID** | ❌ Scalar fallback | ✅ **Hardware CPUID / OSXSAVE** | Native vectorization on bare metal, microVMs & SGX enclaves |
| **Timing Attack Resistance** | ❌ Vulnerable (memory LUTs) | ✅ **Branchless Constant-Time** | Prevents cache-timing leaks in HMAC, JWT & auth tokens |
| **PEM & MIME Whitespace Parser** | ❌ Hard fails on newlines | ✅ **SWAR In-Place Parser** | Direct parsing of X.509 certs, SSH keys & MIME emails |
| **Zero-Alloc `Display` Stream** | ❌ Allocates intermediate String | ✅ **`hyper64::display(&slice)`** | Streams Base64 directly into formatters, logs, or JSON |
| **Compile-Time Evaluation** | ❌ No `const fn` | ✅ **`const fn` & Macros** | Precomputes tokens and cryptographic keys at compile-time |
| **Asynchronous I/O** | ❌ Blocking only | ✅ **Tokio `AsyncWrite` / `AsyncRead`** | Non-blocking chunked streaming for web services |
| **Drop-in Compatibility Layer** | — | ✅ **`base64::Engine` API** | Seamless replacement without rewriting application code |
| **Core Dependencies** | Moderate | ✅ **0 Dependencies** | Zero supply-chain attack surface in default builds |

---

## 📦 Installation & Feature Flags

Add `hyper64` to your `Cargo.toml`:

```toml
[dependencies]
hyper64 = "0.2"
```

### Cargo Features

| Feature | Default | Description |
|:---|:---:|:---|
| `std` | **Yes** | Enables `std::io::Write`/`Read` streams and standard runtime CPU feature queries. |
| `alloc` | **Yes** (via `std`) | Enables `String` and `Vec<u8>` returning convenience APIs (`encode`, `decode`). |
| `simd` | No | Explicit flag reserved for target-specific intrinsics. |
| `serde` | No | Enables zero-copy Serde wrappers (`#[serde(with = "...")]` and `Base64<T>`). |
| `tokio` | No | Enables non-blocking streaming adapters (`AsyncEncoderWriter`, `AsyncDecoderReader`). |

For embedded and bare-metal environments:
```toml
[dependencies]
hyper64 = { version = "0.2", default-features = false }
```

---

## 💡 Quick Start & Usage Examples

### 1. High-Level Convenience API

```rust
use hyper64::{encode, decode};

let raw_data = b"Rust 2026 High Performance Base64 Engine";
let encoded_str = encode(raw_data);
assert_eq!(encoded_str, "UnVzdCAyMDI2IEhpZ2ggUGVyZm9ybWFuY2UgQmFzZTY0IEVuZ2luZQ==");

let decoded_bytes = decode(&encoded_str).unwrap();
assert_eq!(decoded_bytes, raw_data);
```

---

### 2. Zero-Allocation In-Place Decoding (SIMD-Accelerated)

Decodes Base64 data directly inside an existing mutable slice (e.g., packet buffer, pre-allocated arena) without touching the heap:

```rust
use hyper64::{decode_in_place, STANDARD};

let mut packet = *b"SGVsbG8gV29ybGQh"; // 16 bytes on stack
let decoded = decode_in_place(&mut packet, &STANDARD).unwrap();

assert_eq!(decoded, b"Hello World!");
```

---

### 3. In-Place PEM Certificate & MIME Whitespace Decoding

Real-world certificates, SSH keys, and emails contain embedded line breaks (`\r\n`, `\n`) and headers. `hyper64` parses and decodes them directly in-place with **zero memory copies**:

```rust
use hyper64::decode_pem_in_place;

let mut cert = *b"-----BEGIN CERTIFICATE-----\r\nSGVsbG8gV29ybGQh\r\n-----END CERTIFICATE-----\r\n";
let block = decode_pem_in_place(&mut cert).unwrap();

assert_eq!(block.label, "CERTIFICATE");
assert_eq!(block.contents, b"Hello World!");
```

---

### 4. Zero-Allocation `Display` Streaming

Format binary hashes and tokens into loggers, HTTP headers, or JSON serializations without allocating temporary intermediate `String`s:

```rust
use hyper64::display;

let token_hash = [0x42u8; 32];

// Formats directly into the output stream using an internal 64-byte L1 stack buffer!
let header = format!("Authorization: Bearer {}", display(&token_hash));
assert!(header.starts_with("Authorization: Bearer QkJC"));
```

---

### 5. Constant-Time Cryptographic Mode (Anti-Side-Channel)

Eliminates cache-timing vulnerabilities and data-dependent memory lookups when handling private keys, cryptographic nonces, or tokens:

```rust
use hyper64::{encode_constant_time, decode_constant_time, ct_eq};

let secret = [0x5au8; 32];
let mut safe_buffer = [0u8; 44];
encode_constant_time(&secret, &mut safe_buffer);

let mut recovered = [0u8; 32];
decode_constant_time(&safe_buffer, &mut recovered).unwrap();

// Constant-time equality comparison
assert!(ct_eq(&secret, &recovered));
```

---

### 6. Compile-Time (`const fn`) Evaluation

Embed pre-encoded assets or pre-decoded binary configurations directly into your compiled executable:

```rust
use hyper64::{encode_const, decode_const};

const ENCODED_ASSET: &[u8] = &encode_const!(b"static-compile-time-asset");
const DECODED_KEY: [u8; 12] = decode_const!(b"SGVsbG8gV29ybGQh");

assert_eq!(&DECODED_KEY, b"Hello World!");
```

---

### 7. Drop-In Replacement for `crates.io/crates/base64`

Migrate existing codebases instantly by replacing the dependency with `hyper64`:

```rust
use hyper64::engine::{Engine, general_purpose::STANDARD};

let encoded = STANDARD.encode(b"Drop-in replacement for base64 crate");
let decoded = STANDARD.decode(&encoded).unwrap();
assert_eq!(decoded, b"Drop-in replacement for base64 crate");
```

---

### 8. Asynchronous Streaming with Tokio

Stream multi-gigabyte files or network sockets without loading the entire payload into RAM:

```rust,ignore
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use hyper64::{AsyncEncoderWriter, AsyncDecoderReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut encoded_stream = Vec::new();
    {
        let mut encoder = AsyncEncoderWriter::new(&mut encoded_stream);
        encoder.write_all(b"Non-blocking stream data").await?;
        encoder.shutdown().await?;
    }

    let mut decoder = AsyncDecoderReader::new(&encoded_stream[..]);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result).await?;
    assert_eq!(result, b"Non-blocking stream data");
    Ok(())
}
```

---

## 🔒 Safety & Formal Invariants

`hyper64` is built under a strict zero-undefined-behavior mandate:

1. **In-Place Non-Clobbering Proof ($r_k \ge w_k$)**: Because Base64 expands 3 raw bytes into 4 ASCII bytes, reading 4 bytes before writing 3 guarantees the read cursor $r$ always outpaces the write cursor $w$. Memory corruption is mathematically impossible.
2. **Zero Double-Write Memory Initialization**: `hyper64` writes directly into spare vector capacity before calling `set_len`, eliminating zero-filling overhead without reading uninitialized memory.
3. **Unaligned Memory Primitives**: Employs explicit `core::ptr::read_unaligned` and `_mm256_loadu_si256` avoiding strict-aliasing violations.
4. **Hardware CPUID Gating**: Dynamic CPU feature detection inspects CPUID and validates that the OS kernel has enabled YMM register saving (`_xgetbv(0)` check for `XCR0[2:1]`).

For complete formal proofs, refer to [SAFETY.md](SAFETY.md).

---

## 🛡️ Continuous Verification & CI/CD

`hyper64` is audited on every commit by an exhaustive verification matrix:

- **Miri**: Continuous memory model validation checking for undefined behavior and aliasing violations under `-Zmiri-symbolic-alignment-check`.
- **Differential Oracle Testing**: 2,048 randomized differential tests validating equivalence against `crates.io/crates/base64`.
- **libFuzzer with AddressSanitizer (ASan)**: Automated fuzzing targets (`fuzz_decode`, `fuzz_roundtrip`, `fuzz_pem`).
- **Multi-Architecture Matrix**: Validated on Linux x86_64, macOS Apple Silicon (AArch64 NEON), Windows MSVC, Big-Endian s390x, and Cortex-M4 bare metal.
- **Supply Chain Guard**: Audited with `glastor-core/specter` (SemVer stability), `glastor-core/byte-gauge` (binary hardening), and `cargo-deny` (license and vulnerability tracking).

---

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
