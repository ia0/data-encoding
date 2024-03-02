use base64::prelude::{Engine, BASE64_STANDARD};
use base64::DecodeError::*;
use data_encoding::DecodeKind::*;
use data_encoding::{DecodeError, BASE64};

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
}

#[test]
fn difference() {
    let x = b"AAB=";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 2, kind: Trailing });
    assert_eq!(BASE64_STANDARD.decode(x).err().unwrap(), InvalidLastSymbol(2, b'B'));
    let x = b"AA\nB=";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 4, kind: Length });
    assert_eq!(BASE64_STANDARD.decode(x).err().unwrap(), InvalidByte(2, b'\n'));
    let x = b"AAB";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 0, kind: Length });
    assert_eq!(BASE64_STANDARD.decode(x).err().unwrap(), InvalidPadding);
    let x = b"AAA";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 0, kind: Length });
    assert_eq!(BASE64_STANDARD.decode(x).err().unwrap(), InvalidPadding);
    let x = b"A\rA\nB=";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 4, kind: Length });
    assert_eq!(BASE64_STANDARD.decode(x).err().unwrap(), InvalidByte(1, b'\r'));
    let x = b"-_\r\n";
    assert_eq!(BASE64.decode(x).err().unwrap(), DecodeError { position: 0, kind: Symbol });
    assert_eq!(BASE64_STANDARD.decode(x).err().unwrap(), InvalidByte(0, b'-'));
    let x = b"AA==AA==";
    assert_eq!(BASE64.decode(x).unwrap(), vec![0, 0]);
    assert_eq!(BASE64_STANDARD.decode(x).err().unwrap(), InvalidByte(2, b'='));
}
