use base64::Engine;
use hyper64::{
    DecoderReader, EncoderWriter, STANDARD, URL_SAFE_NO_PAD, decode, decode_constant_time,
    decode_in_place, decode_permissive, display, encode,
};
use std::io::{Read, Write};

#[test]
fn test_oracle_equivalence_against_base64_crate() {
    // Test across hundreds of varying lengths and byte patterns
    for len in 0..2048 {
        let raw: Vec<u8> = (0..len).map(|i| ((i * 73 + 19) % 256) as u8).collect();

        // 1. Standard encode equivalence
        let hyper_enc = encode(&raw);
        let base64_enc = base64::prelude::BASE64_STANDARD.encode(&raw);
        assert_eq!(hyper_enc, base64_enc, "Encode mismatch at len {}", len);

        // 2. Standard decode equivalence
        let hyper_dec = decode(&hyper_enc).expect("hyper64 decode failed");
        let base64_dec = base64::prelude::BASE64_STANDARD
            .decode(&base64_enc)
            .expect("base64 crate decode failed");
        assert_eq!(hyper_dec, raw, "hyper64 decode mismatch at len {}", len);
        assert_eq!(
            base64_dec, raw,
            "base64 crate decode mismatch at len {}",
            len
        );

        // 3. In-place decode equivalence
        let mut in_place_buf = hyper_enc.as_bytes().to_vec();
        let in_place_res =
            decode_in_place(&mut in_place_buf, &STANDARD).expect("in-place decode failed");
        assert_eq!(in_place_res, &raw[..], "In-place mismatch at len {}", len);

        // 4. URL-safe unpadded equivalence
        let hyper_url = hyper64::encode_config(&raw, &URL_SAFE_NO_PAD);
        let base64_url = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(&raw);
        assert_eq!(
            hyper_url, base64_url,
            "URL-safe encode mismatch at len {}",
            len
        );

        let hyper_url_dec =
            hyper64::decode_config(&hyper_url, &URL_SAFE_NO_PAD).expect("URL-safe decode failed");
        assert_eq!(
            hyper_url_dec, raw,
            "URL-safe decode mismatch at len {}",
            len
        );

        // 5. Zero-alloc Display formatter equivalence
        let disp_str = format!("{}", display(&raw));
        assert_eq!(disp_str, base64_enc, "Display mismatch at len {}", len);

        // 6. Constant-time decode equivalence (for padded inputs)
        if !hyper_enc.is_empty() {
            let mut ct_out = vec![0u8; raw.len()];
            let written = decode_constant_time(hyper_enc.as_bytes(), &mut ct_out)
                .expect("Constant-time decode failed");
            assert_eq!(
                &ct_out[..written],
                &raw[..],
                "Constant-time decode mismatch at len {}",
                len
            );
        }
    }
}

#[test]
fn test_streaming_io_roundtrip() {
    for chunk_size in [1, 2, 3, 5, 7, 16, 64, 512, 1024] {
        let raw: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();

        // Streaming write
        let mut encoded_stream = Vec::new();
        {
            let mut writer = EncoderWriter::new(&mut encoded_stream);
            for chunk in raw.chunks(chunk_size) {
                writer.write_all(chunk).unwrap();
            }
            writer.finish().unwrap();
        }

        let expected_str = base64::prelude::BASE64_STANDARD.encode(&raw);
        assert_eq!(
            std::str::from_utf8(&encoded_stream).unwrap(),
            expected_str,
            "EncoderWriter streaming failed with chunk_size {}",
            chunk_size
        );

        // Streaming read
        let mut reader = DecoderReader::new(&encoded_stream[..]);
        let mut decoded_stream = Vec::new();
        let mut read_buf = vec![0u8; chunk_size];
        loop {
            let n = reader.read(&mut read_buf).unwrap();
            if n == 0 {
                break;
            }
            decoded_stream.extend_from_slice(&read_buf[..n]);
        }
        assert_eq!(
            decoded_stream, raw,
            "DecoderReader streaming failed with chunk_size {}",
            chunk_size
        );
    }
}

#[test]
fn test_permissive_random_whitespace_injection() {
    let raw = b"High-throughput zero-allocation Base64 engine in Rust 2026!";
    let clean_b64 = encode(raw);

    // Randomly insert whitespaces into clean_b64
    let mut dirty_b64 = Vec::new();
    let whitespaces = [b' ', b'\t', b'\r', b'\n'];
    for (i, &b) in clean_b64.as_bytes().iter().enumerate() {
        dirty_b64.push(b);
        if i % 3 == 0 {
            dirty_b64.push(whitespaces[i % whitespaces.len()]);
        }
        if i % 7 == 0 {
            dirty_b64.push(b'\r');
            dirty_b64.push(b'\n');
        }
    }

    let decoded = decode_permissive(&dirty_b64, &STANDARD).expect("Permissive decode failed");
    assert_eq!(
        &decoded, raw,
        "Permissive decode did not match original data"
    );

    let mut in_place_dirty = dirty_b64.clone();
    let in_place_decoded = hyper64::decode_permissive_in_place(&mut in_place_dirty, &STANDARD)
        .expect("In-place permissive decode failed");
    assert_eq!(in_place_decoded, raw);
}
