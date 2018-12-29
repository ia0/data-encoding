#![feature(test)]

extern crate data_encoding;
extern crate test;

use data_encoding::{Specification, BASE32, BASE32_DNSCURVE, BASE64, BASE64_NOPAD, HEXLOWER};
use test::Bencher;

#[bench]
fn base02_encode_base(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 32768];
    let mut spec = Specification::new();
    spec.symbols.push_str("01");
    let base = spec.encoding().unwrap();
    b.iter(|| base.encode_mut(input, output));
}

#[bench]
fn base02_decode_base(b: &mut Bencher) {
    let input = &[b'0'; 4096];
    let output = &mut [0u8; 512];
    let mut spec = Specification::new();
    spec.symbols.push_str("01");
    let base = spec.encoding().unwrap();
    b.iter(|| base.decode_mut(input, output));
}

#[bench]
fn base04_encode_base(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 16384];
    let mut spec = Specification::new();
    spec.symbols.push_str("0123");
    let base = spec.encoding().unwrap();
    b.iter(|| base.encode_mut(input, output));
}

#[bench]
fn base04_decode_base(b: &mut Bencher) {
    let input = &[b'0'; 4096];
    let output = &mut [0u8; 1024];
    let mut spec = Specification::new();
    spec.symbols.push_str("0123");
    let base = spec.encoding().unwrap();
    b.iter(|| base.decode_mut(input, output));
}

#[bench]
fn base08_encode_base(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 10923];
    let mut spec = Specification::new();
    spec.symbols.push_str("01234567");
    let base = spec.encoding().unwrap();
    b.iter(|| base.encode_mut(input, output));
}

#[bench]
fn base08_decode_base(b: &mut Bencher) {
    let input = &[b'0'; 4096];
    let output = &mut [0u8; 1536];
    let mut spec = Specification::new();
    spec.symbols.push_str("01234567");
    let base = spec.encoding().unwrap();
    b.iter(|| base.decode_mut(input, output));
}

#[bench]
fn base16_encode_base(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 8192];
    b.iter(|| HEXLOWER.encode_mut(input, output));
}

#[bench]
fn base16_decode_base(b: &mut Bencher) {
    let input = &[b'0'; 4096];
    let output = &mut [0u8; 2048];
    b.iter(|| HEXLOWER.decode_mut(input, output));
}

#[bench]
fn base32_encode_base(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 6560];
    b.iter(|| BASE32.encode_mut(input, output));
}

#[bench]
fn base32_decode_base(b: &mut Bencher) {
    let input = &[b'A'; 4096];
    let output = &mut [0u8; 2560];
    b.iter(|| BASE32.decode_mut(input, output));
}

#[bench]
fn base64_encode_base(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 5462];
    b.iter(|| BASE64_NOPAD.encode_mut(input, output));
}

#[bench]
fn base64_decode_base(b: &mut Bencher) {
    let input = &[b'A'; 4096];
    let output = &mut [0u8; 3072];
    b.iter(|| BASE64_NOPAD.decode_mut(input, output));
}

#[bench]
fn base64_encode_pad(b: &mut Bencher) {
    let input = &mut [b'A'; 4096];
    let output = &mut [0u8; 5464];
    b.iter(|| BASE64.encode_mut(input, output));
}

#[bench]
fn base64_decode_pad(b: &mut Bencher) {
    let input = &mut [b'A'; 4096];
    for i in 0 .. 20 {
        let x = 4096 * i / 20 / 4 * 4;
        input[x + 3] = b'=';
        if i % 2 == 0 {
            input[x + 2] = b'=';
        }
    }
    let output = &mut [0u8; 3072];
    b.iter(|| BASE64.decode_mut(input, output).unwrap());
}

#[bench]
fn base64_encode_wrap(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 5608];
    let mut spec = BASE64.specification();
    spec.wrap.width = 76;
    spec.wrap.separator.push_str("\r\n");
    let base64 = spec.encoding().unwrap();
    b.iter(|| base64.encode_mut(input, output));
}

#[bench]
fn base64_decode_wrap(b: &mut Bencher) {
    let input = &mut [b'A'; 4096];
    for i in 0 .. 20 {
        let x = 4096 * i / 20 / 4 * 4;
        input[x + 3] = b'\n';
    }
    let output = &mut [0u8; 3072];
    let mut spec = BASE64.specification();
    spec.wrap.width = 76;
    spec.wrap.separator.push_str("\r\n");
    let base64 = spec.encoding().unwrap();
    b.iter(|| base64.decode_mut(input, output).unwrap());
}

#[bench]
fn dnscurve_decode_base(b: &mut Bencher) {
    let input = &[b'0'; 4096];
    let output = &mut [0u8; 2560];
    b.iter(|| BASE32_DNSCURVE.decode_mut(input, output));
}

#[bench]
fn dnscurve_encode_base(b: &mut Bencher) {
    let input = &[0u8; 4096];
    let output = &mut [0u8; 6554];
    b.iter(|| BASE32_DNSCURVE.encode_mut(input, output));
}
