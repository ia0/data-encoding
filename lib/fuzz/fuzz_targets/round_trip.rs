#![no_main]

#[macro_use]
extern crate libfuzzer_sys;
extern crate data_encoding_fuzz;

use data_encoding_fuzz::{decode_prefix, generate_encoding};

fuzz_target!(|data: &[u8]| {
    let mut data = data;
    let e = generate_encoding(&mut data);
    assert_eq!(e.specification().encoding().unwrap(), e);
    assert_eq!(e.decode(e.encode(data).as_bytes()).unwrap(), data);
    if e.is_canonical() {
        let raw = decode_prefix(&e, &mut data);
        assert_eq!(e.encode(&raw).as_bytes(), data);
    }
});
