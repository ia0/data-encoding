#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate data_encoding;
extern crate data_encoding_fuzz;

use data_encoding::Encoding;
use data_encoding_fuzz::generate_encoding;

fuzz_target!(|data: &[u8]| {
    fuzz_round_trip(data);
});

fn fuzz_round_trip(mut data: &[u8]) -> Option<()> {
    let encoding = generate_encoding(&mut data)?;
    fuzz_encode_decode(&encoding, data);
    if encoding.is_canonical() {
        fuzz_decode_encode_canonical(&encoding, data)
    } else {
        fuzz_decode_encode_decode(&encoding, data)
    }
}

fn fuzz_encode_decode(e: &Encoding, x: &[u8]) {
    assert_eq!(e.decode(e.encode(x).as_bytes()).unwrap(), x);
}

fn fuzz_decode_encode_canonical(e: &Encoding, x: &[u8]) -> Option<()> {
    assert_eq!(e.encode(&e.decode(x).ok()?).as_bytes(), x);
    Some(())
}

fn fuzz_decode_encode_decode(e: &Encoding, x: &[u8]) -> Option<()> {
    fuzz_encode_decode(e, &e.decode(x).ok()?);
    Some(())
}
