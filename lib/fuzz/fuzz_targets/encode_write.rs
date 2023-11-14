#![no_main]

use data_encoding_fuzz::generate_encoding;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut data = data;
    let encoding = generate_encoding(&mut data);
    let input = data;
    let mut output = String::new();
    encoding.encode_write(input, &mut output).unwrap();
    let expected = encoding.encode(input);
    assert_eq!(output, expected);
});
