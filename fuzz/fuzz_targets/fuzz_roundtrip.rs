#![no_main]
use libfuzzer_sys::fuzz_target;
use hyper64::{encode, decode, decode_in_place, STANDARD};

fuzz_target!(|data: &[u8]| {
    let encoded = encode(data);
    let decoded = decode(&encoded).expect("Valid encoded Base64 must decode successfully");
    assert_eq!(&decoded[..], data);

    let mut in_place_buf = encoded.as_bytes().to_vec();
    let in_place_decoded = decode_in_place(&mut in_place_buf, &STANDARD)
        .expect("Valid encoded Base64 must decode in-place successfully");
    assert_eq!(in_place_decoded, data);
});

