#![no_main]

use data_encoding_fuzz::generate_encoding;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data = data;
    let encoding = generate_encoding(&mut data);
    let mut output = String::new();
    let mut input = Vec::new();
    let mut encoder = encoding.new_encoder(&mut output);
    while !data.is_empty() {
        let len = std::cmp::min(data[0] as usize, data.len() - 1);
        let chunk = &data[1 ..][.. len];
        input.extend_from_slice(chunk);
        encoder.append(chunk);
        data = &data[1 + len ..];
    }
    encoder.finalize();
    let expected = encoding.encode(&input);
    assert_eq!(output, expected);
});
