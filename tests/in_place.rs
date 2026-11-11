use hyper64::{STANDARD, decode_in_place, encode};

#[test]
fn test_in_place_randomized() {
    let mut data = Vec::with_capacity(1000);
    for i in 0..1000 {
        data.push((i * 37 % 256) as u8);
        let encoded_str = encode(&data);
        let mut in_place_buf = encoded_str.as_bytes().to_vec();

        let decoded =
            decode_in_place(&mut in_place_buf, &STANDARD).expect("In-place decode should succeed");

        assert_eq!(decoded, &data[..], "Failed for length {}", i);
    }
}
