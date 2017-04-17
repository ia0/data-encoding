#![feature(test)]

extern crate test;
extern crate data_encoding;

use data_encoding::BASE64;
use test::Bencher;

#[bench]
fn base64_encode(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 5464];
    b.iter(|| BASE64.encode_mut(input, output));
}

#[bench]
fn base64_decode(b: &mut Bencher) {
    let input = &[b'A'; 4096];
    let output = &mut [0u8; 3072];
    b.iter(|| BASE64.decode_mut(input, output));
}
