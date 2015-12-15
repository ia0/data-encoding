extern crate data_encoding;
extern crate rustc_serialize;
extern crate base64;

#[test]
fn difference() {
    use data_encoding::base64::decode;
    use data_encoding::decode::Error::*;
    use rustc_serialize::base64::FromBase64;
    use base64::u8de;
    let x = b"AAB=";
    assert_eq!(decode(x), Err(BadPadding));
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    assert_eq!(u8de(x).unwrap(), vec![0, 0]);
    let x = b"AAA";
    assert_eq!(decode(x), Err(BadLength));
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    assert_eq!(u8de(x).unwrap(), vec![0, 0]);
    let x = b"A\rA\nB=";
    assert_eq!(decode(x), Err(BadLength));
    assert_eq!(x.from_base64().unwrap(), vec![0, 0]);
    let x = b"-_\r\n";
    assert_eq!(decode(x), Err(BadCharacter(0)));
    assert_eq!(x.from_base64().unwrap(), vec![251]);
}
