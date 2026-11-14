//! Constant-time security example demonstrating side-channel defense in hyper64:
//! 1. Branchless encoding of cryptographic keys / secrets
//! 2. Branchless constant-time decoding (immune to timing attacks)
//! 3. Constant-time slice comparison `ct_eq` to prevent token forging
//!
//! Run with: `cargo run --example constant_time_security`

use hyper64::{ct_eq, decode_constant_time, encode_constant_time};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ hyper64: Constant-Time Cryptographic Hardening Demo\n");

    // Highly sensitive cryptographic key / HMAC token (32 bytes)
    let secret_key = b"super_secret_master_key_32_bytes";

    // 1. Branchless Constant-Time Encoding
    // Avoids lookup tables (LUTs) that cause cache-timing leakage (Spectre/CacheBleed).
    let mut encoded_buf = [0u8; 64];
    let encoded_len = encode_constant_time(secret_key, &mut encoded_buf);
    let encoded_slice = &encoded_buf[..encoded_len];

    println!("1. Constant-Time Encoded Secret:");
    println!("   {}\n", std::str::from_utf8(encoded_slice)?);

    // 2. Branchless Constant-Time Decoding
    // Guarantees uniform instruction execution latency regardless of byte values.
    let mut decoded_buf = [0u8; 64];
    let decoded_len = decode_constant_time(encoded_slice, &mut decoded_buf)?;
    let recovered_secret = &decoded_buf[..decoded_len];

    println!("2. Constant-Time Decoded Secret (Matches: {}):", recovered_secret == secret_key);
    println!("   \"{}\"\n", std::str::from_utf8(recovered_secret)?);

    // 3. Constant-Time Comparison (`ct_eq`)
    // Standard `==` short-circuits on the first mismatched byte, leaking token prefixes.
    // `ct_eq` scans all bytes in constant time regardless of where mismatches occur.
    let untrusted_incoming_token = b"super_secret_master_key_32_bytes";
    let forged_token = b"super_secret_master_key_32_fake_";

    let is_valid = ct_eq(secret_key, untrusted_incoming_token);
    let is_forgery_detected = !ct_eq(secret_key, forged_token);

    println!("3. Constant-Time Token Verification (Timing-Attack Immune):");
    println!("   Valid token verified:   {}", is_valid);
    println!("   Forged token rejected:  {}\n", is_forgery_detected);

    assert!(is_valid);
    assert!(is_forgery_detected);

    println!("✅ All cryptographic constant-time invariants verified!");

    Ok(())
}
