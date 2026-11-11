use hyper64::{STANDARD, URL_SAFE_NO_PAD, decode, decode_in_place, encode};

/// Official RFC 4648 Section 10 Test Vectors:
/// BASE64("") = ""
/// BASE64("f") = "Zg=="
/// BASE64("fo") = "Zm8="
/// BASE64("foo") = "Zm9v"
/// BASE64("foob") = "Zm9vYg=="
/// BASE64("fooba") = "Zm9vYmE="
/// BASE64("foobar") = "Zm9vYmFy"
const RFC_VECTORS: &[(&[u8], &str)] = &[
    (b"", ""),
    (b"f", "Zg=="),
    (b"fo", "Zm8="),
    (b"foo", "Zm9v"),
    (b"foob", "Zm9vYg=="),
    (b"fooba", "Zm9vYmE="),
    (b"foobar", "Zm9vYmFy"),
];

#[test]
fn test_rfc4648_encoding() {
    for &(input, expected) in RFC_VECTORS {
        let encoded = encode(input);
        assert_eq!(encoded, expected, "Encoding failed for {:?}", input);
    }
}

#[test]
fn test_rfc4648_decoding() {
    for &(expected, encoded) in RFC_VECTORS {
        let decoded = decode(encoded).expect("Decoding failed");
        assert_eq!(&decoded[..], expected, "Decoding mismatch for {}", encoded);
    }
}

#[test]
fn test_rfc4648_in_place_decoding() {
    for &(expected, encoded) in RFC_VECTORS {
        let mut buf = encoded.as_bytes().to_vec();
        let decoded = decode_in_place(&mut buf, &STANDARD).expect("In-place decoding failed");
        assert_eq!(decoded, expected, "In-place mismatch for {}", encoded);
    }
}

#[test]
fn test_url_safe_no_pad() {
    let input = b"\xFB\xFF\xBF";
    let encoded = hyper64::encode_config(input, &URL_SAFE_NO_PAD);
    assert_eq!(encoded, "-_-_");

    let decoded = hyper64::decode_config(&encoded, &URL_SAFE_NO_PAD).unwrap();
    assert_eq!(&decoded[..], input);
}

#[test]
fn test_invalid_padding_rejected() {
    // RFC 4648 §3.5: Non-zero unused bits in padding must be rejected (strict check)
    // "Zh==" has non-zero bits in the unused sextet
    let res = decode("Zh==");
    assert!(res.is_err(), "Non-zero padding bits should be rejected");
}
