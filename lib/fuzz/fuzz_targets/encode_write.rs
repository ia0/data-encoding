#![no_main]

use data_encoding_fuzz::{generate_encoding, generate_usize};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data = data;
    let encoding = generate_encoding(&mut data);
    let mut buffer = vec![0; generate_usize(&mut data, 510, 2050)];
    let input = data;
    let mut output = String::new();
    encoding.encode_write_buffer(input, &mut output, &mut buffer).unwrap();
    let expected = encoding.encode(input);
    assert_eq!(output, expected);
});
