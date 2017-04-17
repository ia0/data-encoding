extern crate data_encoding;

use data_encoding::{Builder, NoPad, Padded, DecodeError};
use data_encoding::DecodeKind::*;

macro_rules! test {
    (fn $t: ident; $b: ident; $($s: stmt);*;) => {
        #[test]
        fn $t() {
            fn test(b: &$b, x: &[u8], y: &[u8]) {
                assert_eq!(&b.encode(x).into_bytes() as &[u8], y);
                assert_eq!(&b.decode(y).unwrap() as &[u8], x);
            }
            $($s);*
        }
    };
}

fn errmsg<T, E: std::error::Error>(x: Result<T, E>) -> String {
    format!("{}", x.err().unwrap())
}

test!{
    fn base2; NoPad;
    let b = Builder::new(b"01").no_pad().unwrap();
    test(&b, b"", b"");
    test(&b, b"f", b"01100110");
    test(&b, b"fo", b"0110011001101111");
    test(&b, b"foo", b"011001100110111101101111");
}

test!{
    fn base4; NoPad;
    let b = Builder::new(b"0123").no_pad().unwrap();
    test(&b, b"", b"");
    test(&b, b"f", b"1212");
    test(&b, b"fo", b"12121233");
    test(&b, b"foo", b"121212331233");
    test(&b, b"foob", b"1212123312331202");
    test(&b, b"fooba", b"12121233123312021201");
    test(&b, b"foobar", b"121212331233120212011302");
}

test!{
    fn base8; Padded;
    let b = Builder::new(b"01234567").pad(b'=').padded().unwrap();
    test(&b, b"", b"");
    test(&b, b"f", b"314=====");
    test(&b, b"fo", b"314674==");
    test(&b, b"foo", b"31467557");
    test(&b, b"foob", b"31467557304=====");
    test(&b, b"fooba", b"31467557304604==");
    test(&b, b"foobar", b"3146755730460562");
}

test!{
    fn base16; NoPad;
    let b = &data_encoding::HEXUPPER;
    test(b, b"", b"");
    test(b, b"f", b"66");
    test(b, b"fo", b"666F");
    test(b, b"foo", b"666F6F");
    test(b, b"foob", b"666F6F62");
    test(b, b"fooba", b"666F6F6261");
    test(b, b"foobar", b"666F6F626172");
}

test!{
    fn base32; Padded;
    let b = &data_encoding::BASE32;
    test(b, b"", b"");
    test(b, b"f", b"MY======");
    test(b, b"fo", b"MZXQ====");
    test(b, b"foo", b"MZXW6===");
    test(b, b"foob", b"MZXW6YQ=");
    test(b, b"fooba", b"MZXW6YTB");
    test(b, b"foobar", b"MZXW6YTBOI======");
}

test!{
    fn base32hex; Padded;
    let b = &data_encoding::BASE32HEX;
    test(b, b"", b"");
    test(b, b"f", b"CO======");
    test(b, b"fo", b"CPNG====");
    test(b, b"foo", b"CPNMU===");
    test(b, b"foob", b"CPNMUOG=");
    test(b, b"fooba", b"CPNMUOJ1");
    test(b, b"foobar", b"CPNMUOJ1E8======");
}

test!{
    fn base64; Padded;
    let b = &data_encoding::BASE64;
    test(b, b"", b"");
    test(b, b"f", b"Zg==");
    test(b, b"fo", b"Zm8=");
    test(b, b"foo", b"Zm9v");
    test(b, b"foob", b"Zm9vYg==");
    test(b, b"fooba", b"Zm9vYmE=");
    test(b, b"foobar", b"Zm9vYmFy");
}

test!{
    fn base64_no_pad; NoPad;
    let b = data_encoding::BASE64.no_pad();
    test(b, b"", b"");
    test(b, b"f", b"Zg");
    test(b, b"fo", b"Zm8");
    test(b, b"foo", b"Zm9v");
    test(b, b"foob", b"Zm9vYg");
    test(b, b"fooba", b"Zm9vYmE");
    test(b, b"foobar", b"Zm9vYmFy");
}

#[test]
fn base32_error() {
    let b = &data_encoding::BASE32;
    assert_eq!(b.decode(b"ABC").err().unwrap(),
               DecodeError { position: 0, kind: Length });
    assert_eq!(b.decode(b"========").err().unwrap(),
               DecodeError { position: 0, kind: Padding });
    assert_eq!(b.decode(b"MB======").err().unwrap(),
               DecodeError { position: 1, kind: Trailing });
    assert_eq!(b.decode(b"MA===AAA").err().unwrap(),
               DecodeError { position: 2, kind: Symbol });
    assert_eq!(b.decode(b"MAA=====").err().unwrap(),
               DecodeError { position: 3, kind: Padding });
    assert_eq!(b.decode(b"MAABBB==").err().unwrap(),
               DecodeError { position: 6, kind: Padding });
    assert_eq!(b.decode(b"MA======MA======").err().unwrap(),
               DecodeError { position: 2, kind: Symbol });
}

#[test]
fn base64_error() {
    let b = &data_encoding::BASE64;
    assert_eq!(b.decode(b"====").err().unwrap(),
               DecodeError { position: 0, kind: Padding });
    assert_eq!(b.decode_concat(b"====").err().unwrap(),
               DecodeError { position: 0, kind: Padding });
    assert_eq!(b.decode(b"Zm9vYmFy====").err().unwrap(),
               DecodeError { position: 8, kind: Padding });
    assert_eq!(b.decode_concat(b"Zm9vYmFy====").err().unwrap(),
               DecodeError { position: 8, kind: Padding });
    assert_eq!(b.decode_concat(b"YmE=Zg==Zg==").unwrap(), b"baff");
    assert_eq!(b.decode(b"Zm9vYmFy----").err().unwrap(),
               DecodeError { position: 8, kind: Symbol });
    assert_eq!(b.decode_concat(b"YmE=-mFyZg==").err().unwrap(),
               DecodeError { position: 4, kind: Symbol });
    assert_eq!(b.decode_concat(b"YmE=-g==Zg==").err().unwrap(),
               DecodeError { position: 4, kind: Symbol });
    assert_eq!(b.decode_concat(b"YmE=Z-==Zg==").err().unwrap(),
               DecodeError { position: 5, kind: Symbol });
    assert_eq!(b.decode_concat(b"YmE=Y-FyZg==").err().unwrap(),
               DecodeError { position: 5, kind: Symbol });
    assert_eq!(b.decode_concat(b"YmE=Z===Zg==").err().unwrap(),
               DecodeError { position: 5, kind: Padding });
    assert_eq!(b.decode_concat(b"YmE=Zh==Zg==").err().unwrap(),
               DecodeError { position: 5, kind: Trailing });
    assert_eq!(b.decode_len(4).unwrap(), 3);
    assert_eq!(b.decode_len(5).err().unwrap(),
               DecodeError { position: 4, kind: Length });
    assert_eq!(b.decode_len(6).err().unwrap(),
               DecodeError { position: 4, kind: Length });
    assert_eq!(b.decode_len(7).err().unwrap(),
               DecodeError { position: 4, kind: Length });
}

#[test]
fn base64_nopad_error() {
    let b = data_encoding::BASE64.no_pad();
    assert_eq!(b.decode(b"Z").err().unwrap(),
               DecodeError { position: 0, kind: Length });
    assert_eq!(b.decode(b"Zh").err().unwrap(),
               DecodeError { position: 1, kind: Trailing });
    assert_eq!(b.decode(b"Zg==").err().unwrap(),
               DecodeError { position: 2, kind: Symbol });
    assert_eq!(b.decode_len(4).unwrap(), 3);
    assert_eq!(b.decode_len(5).err().unwrap(),
               DecodeError { position: 4, kind: Length });
    assert_eq!(b.decode_len(6).unwrap(), 4);
    assert_eq!(b.decode_len(7).unwrap(), 5);
}

#[test]
fn bit_order() {
    let mut builder = Builder::new(b"0123456789abcdef");
    let msb = builder.no_pad().unwrap();
    builder.least_significant_bit_first();
    let lsb = builder.no_pad().unwrap();
    assert_eq!(msb.encode(b"ABC"), "414243");
    assert_eq!(lsb.encode(b"ABC"), "142434");
}

#[test]
fn trailing_bits() {
    let mut builder = Builder::new(b"01234567");
    let strict = builder.no_pad().unwrap();
    let permissive = builder.ignore_trailing_bits().no_pad().unwrap();
    assert!(strict.decode(b"001").is_err());
    assert!(permissive.decode(b"001").is_ok());
}

#[test]
fn dns_curve() {
    let base = Builder::new(b"0123456789bcdfghjklmnpqrstuvwxyz")
        .translate(b"BCDFGHJKLMNPQRSTUVWXYZ", b"bcdfghjklmnpqrstuvwxyz")
        .least_significant_bit_first().no_pad().unwrap();
    assert_eq!(base.encode(&[0x64, 0x88]), "4321");
    assert_eq!(base.decode(b"4321").unwrap(), vec![0x64, 0x88]);
    assert!(base.decode(b"4322").is_err());
    assert!(base.decode(b"4324").is_err());
    assert!(base.decode(b"4328").is_err());
    assert!(base.decode(b"432j").is_err());
    assert!(base.decode(b"08").is_err());
    assert!(base.decode(b"0000j").is_err());
    assert!(base.decode(b"0000004").is_err());
    assert_eq!(base.encode(b"f"), "63");
    assert_eq!(base.encode(b"fo"), "6vv0");
    assert_eq!(base.encode(b"foo"), "6vvy6");
    assert_eq!(base.encode(b"foob"), "6vvy6k1");
    assert_eq!(base.encode(b"fooba"), "6vvy6k5d");
    assert_eq!(base.encode(b"foobar"), "6vvy6k5dl3");
    assert_eq!(base.decode(b"6VVY6K5DL3").unwrap(), b"foobar");
}

#[test]
fn builder() {
    assert_eq!(errmsg(Builder::new(&[0u8] as &[u8]).no_pad()),
               "invalid number of symbols");
    assert_eq!(errmsg(Builder::new(&[0u8, 128] as &[u8]).no_pad()),
               "non-ascii symbol 0x80");
    assert_eq!(errmsg(Builder::new(b"01").pad(b' ').no_pad()),
               "unnecessary or missing padding");
    assert_eq!(errmsg(Builder::new(b"01").pad(b' ').padded()),
               "unnecessary or missing padding");
    assert_eq!(errmsg(Builder::new(b"01").translate(b"1", b" ").no_pad()),
               "invalid value for '1'");
    let mut builder = Builder::new(b"01");
    builder.values[b'0' as usize] = 2;
    assert_eq!(errmsg(builder.no_pad()), "invalid value for '0'");
    let mut builder = Builder::new(b"01");
    builder.values[b' ' as usize] = 2;
    assert_eq!(errmsg(builder.no_pad()), "invalid value for ' '");
    let mut builder = Builder::new(b"01234567");
    builder.padding = Some(128);
    assert_eq!(errmsg(builder.padded()), "non-ascii padding 0x80");
    builder.padding = Some(b'0');
    assert_eq!(errmsg(builder.padded()), "padding symbol conflict");
}

#[test]
fn decode_error() {
    let b = &data_encoding::BASE64;
    assert_eq!(errmsg(b.decode(b"A")), "invalid length at 0");
    assert_eq!(errmsg(b.decode(b"A.AA")), "invalid symbol at 1");
    assert_eq!(errmsg(b.decode(b"AAB=")), "non-zero trailing bits at 2");
    assert_eq!(errmsg(b.decode(b"A===")), "invalid padding length at 1");
}
