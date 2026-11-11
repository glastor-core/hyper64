#![no_main]
use libfuzzer_sys::fuzz_target;
use hyper64::{decode_pem_in_place, decode_permissive_in_place, STANDARD};

fuzz_target!(|data: &[u8]| {
    let mut buf = data.to_vec();
    let _ = decode_permissive_in_place(&mut buf, &STANDARD);

    let mut pem_buf = data.to_vec();
    let _ = decode_pem_in_place(&mut pem_buf);
});

