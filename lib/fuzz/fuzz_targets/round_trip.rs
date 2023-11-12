#![no_main]

use data_encoding_fuzz::{decode_prefix, generate_encoding};
use libfuzzer_sys::fuzz_target;

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
