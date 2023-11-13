#![no_main]

use data_encoding_fuzz::{generate_bytes, generate_encoding, generate_usize};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data = data;
    let encoding = generate_encoding(&mut data);
    let mut output = String::new();
    let mut input = Vec::new();
    let mut encoder = encoding.new_encoder(&mut output);
    while !data.is_empty() {
        let len = generate_usize(&mut data, 0, 3 * 256 - 1);
        let chunk = generate_bytes(&mut data, len);
        input.extend_from_slice(chunk);
        encoder.append(chunk);
    }
    encoder.finalize();
    let expected = encoding.encode(&input);
    assert_eq!(output, expected);
});
