#![feature(test)]

extern crate base64;
extern crate cmp;
extern crate data_encoding;
extern crate test;

use data_encoding::BASE64;
use test::Bencher;

fn encode_mut<F: Fn(&[u8], &mut [u8])>(b: &mut Bencher, f: F) {
    let input = &[0u8; 1024 * 3];
    let output = &mut [0u8; 1024 * 4];
    b.iter(|| f(input, output));
}

fn encode<F: Fn(&[u8]) -> String>(b: &mut Bencher, f: F) {
    let input = &[0u8; 1024 * 3];
    b.iter(|| f(input));
}

fn decode_mut<E, T, F: Fn(&[u8], &mut [u8]) -> Result<T, E>>(b: &mut Bencher, f: F) {
    let input = &[b'A'; 1024 * 4];
    let output = &mut [0u8; 1024 * 3];
    b.iter(|| f(input, output));
}

fn decode<E, F: Fn(&[u8]) -> Result<Vec<u8>, E>>(b: &mut Bencher, f: F) {
    let input = &[b'A'; 1024 * 4];
    b.iter(|| f(input));
}

#[bench]
fn b00_encode_mut_seq_gcc(b: &mut Bencher) {
    encode_mut(b, cmp::base64_encode_seq_gcc)
}

#[bench]
fn b01_encode_mut_seq_clang(b: &mut Bencher) {
    encode_mut(b, cmp::base64_encode_seq_clang)
}

#[bench]
fn b02_encode_mut_par_clang(b: &mut Bencher) {
    encode_mut(b, cmp::base64_encode_par_clang)
}

#[bench]
fn b03_encode_mut_par_gcc(b: &mut Bencher) {
    encode_mut(b, cmp::base64_encode_par_gcc)
}

#[bench]
fn b04_encode_mut_crate(b: &mut Bencher) {
    let base64_encode = |input: &[u8], output: &mut [u8]| BASE64.encode_mut(input, output);
    encode_mut(b, base64_encode);
}

#[bench]
fn b05_encode_crate(b: &mut Bencher) {
    let base64_encode = |input: &[u8]| BASE64.encode(input);
    encode(b, base64_encode);
}

#[bench]
fn b06_encode_base64(b: &mut Bencher) {
    use base64::Engine;
    encode(b, |x| base64::engine::general_purpose::STANDARD.encode(x));
}

#[bench]
fn b07_encode_vb64(b: &mut Bencher) {
    encode(b, |x| vb64::encode(x));
}

#[bench]
fn b08_decode_mut_seq_gcc(b: &mut Bencher) {
    decode_mut(b, cmp::base64_decode_seq_gcc);
}

#[bench]
fn b09_decode_mut_seq_clang(b: &mut Bencher) {
    decode_mut(b, cmp::base64_decode_seq_clang);
}

#[bench]
fn b10_decode_mut_par_clang(b: &mut Bencher) {
    decode_mut(b, cmp::base64_decode_par_clang);
}

#[bench]
fn b11_decode_mut_par_gcc(b: &mut Bencher) {
    decode_mut(b, cmp::base64_decode_par_gcc);
}

#[bench]
fn b12_decode_mut_crate(b: &mut Bencher) {
    let base64_decode = |input: &[u8], output: &mut [u8]| BASE64.decode_mut(input, output);
    decode_mut(b, base64_decode);
}

#[bench]
fn b13_decode_crate(b: &mut Bencher) {
    let base64_decode = |input: &[u8]| BASE64.decode(input);
    decode(b, base64_decode);
}

#[bench]
fn b14_decode_base64(b: &mut Bencher) {
    use base64::Engine;
    decode(b, |x| base64::engine::general_purpose::STANDARD.decode(x));
}

#[bench]
fn b15_decode_vb64(b: &mut Bencher) {
    decode(b, |x| vb64::decode(x));
}
