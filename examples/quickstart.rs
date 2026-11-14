//! Quickstart example demonstrating hyper64 core operations:
//! 1. Standard allocation-based encoding and decoding
//! 2. Zero-allocation in-place decoding (overwriting input buffer)
//! 3. Zero-allocation `display()` formatter
//! 4. URL-Safe alphabet configuration

use hyper64::{URL_SAFE_NO_PAD, decode, decode_in_place, display, encode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ hyper64: Quickstart & Core APIs\n");

    let payload = b"Hello from GLASTOR hyper64! SIMD-accelerated Base64 engine.";

    // 1. Standard Encoding
    let encoded = encode(payload);
    println!("1. Encoded (Standard):");
    println!("   {}\n", encoded);

    // 2. Standard Decoding
    let decoded = decode(&encoded)?;
    assert_eq!(&decoded[..], payload);
    println!("2. Decoded successfully:");
    println!("   {}\n", String::from_utf8_lossy(&decoded));

    // 3. Zero-Allocation In-Place Decoding (Mathematical Proof in Action)
    // Because Base64 decodes 4 bytes into 3 bytes, the read cursor always
    // stays ahead of the write cursor. hyper64 can decode directly in-place!
    let mut in_place_buffer = encoded.into_bytes();
    let decoded_slice = decode_in_place(&mut in_place_buffer, &hyper64::STANDARD)?;
    assert_eq!(decoded_slice, payload);
    println!("3. Zero-Allocation In-Place Decoding:");
    println!("   Decoded {} bytes with 0 heap allocations!\n", decoded_slice.len());

    // 4. Zero-Allocation `display()` Formatter
    // Formats directly into any std::fmt::Write without allocating intermediate Strings.
    println!("4. Direct Zero-Allocation Formatter:");
    println!("   Display: {}\n", display(payload));

    // 5. URL-Safe Encoding without Padding
    let url_safe_encoded = hyper64::encode_config(payload, &URL_SAFE_NO_PAD);
    println!("5. URL-Safe (No Padding):");
    println!("   {}", url_safe_encoded);

    Ok(())
}
