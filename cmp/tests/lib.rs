extern crate base64;
extern crate cmp;
extern crate data_encoding;
#[macro_use]
extern crate lazy_static;
extern crate rustc_serialize;

use data_encoding::DecodeKind::*;
use data_encoding::{DecodeError, Encoding, Specification, BASE64};
use rustc_serialize::base64::{FromBase64, ToBase64, STANDARD};

#[test]
fn encode_exact() {
    let tests = &[
        (b"" as &[u8], b"" as &[u8]),
        (b"foo" as &[u8], b"Zm9v" as &[u8]),
        (b"foobar" as &[u8], b"Zm9vYmFy" as &[u8]),
    ];
    for &(ref i, ref o) in tests {
        let mut r = vec![0u8; o.len()];
        cmp::base64_encode_seq_gcc(i, &mut r);
        assert_eq!(&r, o);
    }
    for &(ref i, ref o) in tests {
        let mut r = vec![0u8; o.len()];
        cmp::base64_encode_par_gcc(i, &mut r);
        assert_eq!(&r, o);
    }
    for &(ref i, ref o) in tests {
        let mut r = vec![0u8; o.len()];
        BASE64.encode_mut(i, &mut r);
        assert_eq!(&r, o);
    }
    for &(ref i, ref o) in tests {
        assert_eq!(&i.to_base64(STANDARD).as_bytes(), o);
    }
}

#[test]
fn difference() {
    let x = b"AAB=";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 2, kind: Trailing });
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    assert_eq!(base64::decode(x).unwrap(), vec![0, 0]);
    let x = b"AA\nB=";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 4, kind: Length });
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    assert_eq!(base64::decode(x).err().unwrap(), base64::DecodeError::InvalidLength);
    let x = b"AAB";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 0, kind: Length });
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    assert_eq!(base64::decode(x).unwrap(), vec![0, 0]);
    let x = b"AAA";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 0, kind: Length });
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    assert_eq!(base64::decode(x).unwrap(), vec![0, 0]);
    let x = b"A\rA\nB=";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 4, kind: Length });
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    assert_eq!(base64::decode(x).err().unwrap(), base64::DecodeError::InvalidByte(1, b'\r'));
    let x = b"-_\r\n";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 0, kind: Symbol });
    assert_eq!(x.from_base64().unwrap(), vec![251]);
    assert_eq!(base64::decode(x).err().unwrap(), base64::DecodeError::InvalidByte(0, b'-'));
    let x = b"AA==AA==";
    assert_eq!(BASE64.decode(x).unwrap(), vec![0, 0]);
    assert!(x.from_base64().is_err());
    assert_eq!(base64::decode(x).err().unwrap(), base64::DecodeError::InvalidByte(2, b'='));
}

lazy_static! {
    static ref HEX: Encoding = {
        let mut spec = Specification::new();
        spec.symbols.push_str("0123456789abcdef");
        spec.translate.from.push_str("ABCDEF");
        spec.translate.to.push_str("abcdef");
        spec.encoding().unwrap()
    };
}

#[test]
fn lazy_static_hex() {
    assert_eq!(HEX.encode(b"Hello"), "48656c6c6f");
    assert_eq!(HEX.decode(b"48656c6c6f").unwrap(), b"Hello");
    assert_eq!(*HEX, data_encoding::HEXLOWER_PERMISSIVE);
}
