#![no_main]
use libfuzzer_sys::fuzz_target;
use hyper64::{decode, decode_in_place, STANDARD, URL_SAFE_NO_PAD};

fuzz_target!(|data: &[u8]| {
    // Fuzz standard decode with completely unconstrained arbitrary byte slices
    let _ = decode(data);

    // Fuzz in-place decoding
    let mut buf = data.to_vec();
    let _ = decode_in_place(&mut buf, &STANDARD);

    // Fuzz URL-safe decoding
    let mut url_buf = data.to_vec();
    let _ = decode_in_place(&mut url_buf, &URL_SAFE_NO_PAD);
});

