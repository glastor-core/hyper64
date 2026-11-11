use hyper64::{decode, encode, encode_constant_time, encoded_len};

#[test]
fn test_constant_time_matches_standard() {
    for len in 0..500 {
        let input: Vec<u8> = (0..len).map(|x| (x * 79 % 256) as u8).collect();
        let expected = encode(&input);

        let out_len = encoded_len(len, true);
        let mut ct_out = vec![0u8; out_len];
        let written = encode_constant_time(&input, &mut ct_out);

        assert_eq!(written, out_len);
        assert_eq!(core::str::from_utf8(&ct_out).unwrap(), expected);

        let decoded = decode(&ct_out).unwrap();
        assert_eq!(decoded, input);
    }
}
