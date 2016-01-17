#![feature(test)]

extern crate test;
extern crate rustc_serialize;
extern crate data_encoding;

use test::Bencher;
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};
use data_encoding::base64;

fn encode<F: FnMut(&[u8]) -> String>(b: &mut Bencher, mut f: F) {
    let input = &[0u8; 4096];
    b.iter(|| f(input));
}

fn decode<E, F: FnMut(&[u8]) -> Result<Vec<u8>, E>>(b: &mut Bencher, mut f: F) {
    let input = &[b'A'; 4096];
    b.iter(|| f(input));
}

#[bench]
fn encode_crate(b: &mut Bencher) {
    encode(b, base64::encode);
}

#[bench]
fn encode_rustc(b: &mut Bencher) {
    encode(b, |x| x.to_base64(STANDARD));
}

#[bench]
fn decode_crate(b: &mut Bencher) {
    decode(b, base64::decode);
}

#[bench]
fn decode_rustc(b: &mut Bencher) {
    decode(b, |x| x.from_base64());
}
