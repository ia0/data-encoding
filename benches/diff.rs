#![feature(test)]

extern crate test;
extern crate rustc_serialize;
extern crate base64 as extern_base64;
extern crate data_encoding;

use test::Bencher;
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};
use data_encoding::base64;

fn encode<F: FnMut(&[u8]) -> Vec<u8>>(b: &mut Bencher, mut f: F) {
    let input = &[0u8; 4096];
    b.iter(|| f(input));
}

fn decode<E, F: FnMut(&[u8]) -> Result<Vec<u8>, E>>(b: &mut Bencher, mut f: F) {
    let input = &[b'A'; 4096];
    b.iter(|| f(input));
}

#[bench]
fn data_encode(b: &mut Bencher) {
    encode(b, |x| base64::encode(x).into_bytes());
}

#[bench]
fn rustc_encode(b: &mut Bencher) {
    encode(b, |x| x.to_base64(STANDARD).into_bytes());
}

#[bench]
fn base64_encode(b: &mut Bencher) {
    encode(b, |x| extern_base64::u8en(x).unwrap());
}

#[bench]
fn data_decode(b: &mut Bencher) {
    decode(b, base64::decode);
}

#[bench]
fn rustc_decode(b: &mut Bencher) {
    decode(b, |x| x.from_base64());
}

#[bench]
fn base64_decode(b: &mut Bencher) {
    decode(b, extern_base64::u8de);
}
