extern crate data_encoding;

macro_rules! test {
    (fn $b: ident; $($s: stmt);*;) => {
        #[test]
        fn $b() {
            use data_encoding::$b::*;
            #[allow(unused_imports)]
            use data_encoding::decode::Error::*;
            fn test(x: &[u8], y: &[u8]) {
                assert_eq!(&encode(x).into_bytes() as &[u8], y);
                assert_eq!(&decode(y).unwrap() as &[u8], x);
            }
            $($s);*
        }
    };
}

test!{
    fn base2;
    test(b"", b"");
    test(b"f", b"01100110");
    test(b"fo", b"0110011001101111");
    test(b"foo", b"011001100110111101101111");
}

test!{
    fn base4;
    test(b"", b"");
    test(b"f", b"1212");
    test(b"fo", b"12121233");
    test(b"foo", b"121212331233");
    test(b"foob", b"1212123312331202");
    test(b"fooba", b"12121233123312021201");
    test(b"foobar", b"121212331233120212011302");
}

test!{
    fn base8;
    test(b"", b"");
    test(b"f", b"314=====");
    test(b"fo", b"314674==");
    test(b"foo", b"31467557");
    test(b"foob", b"31467557304=====");
    test(b"fooba", b"31467557304604==");
    test(b"foobar", b"3146755730460562");
}

test!{
    fn base16;
    test(b"", b"");
    test(b"f", b"66");
    test(b"fo", b"666F");
    test(b"foo", b"666F6F");
    test(b"foob", b"666F6F62");
    test(b"fooba", b"666F6F6261");
    test(b"foobar", b"666F6F626172");
}

test!{
    fn base32;
    test(b"", b"");
    test(b"f", b"MY======");
    test(b"fo", b"MZXQ====");
    test(b"foo", b"MZXW6===");
    test(b"foob", b"MZXW6YQ=");
    test(b"fooba", b"MZXW6YTB");
    test(b"foobar", b"MZXW6YTBOI======");
    assert_eq!(decode(b"ABC"), Err(BadLength));
    assert_eq!(decode(b"MB======"), Err(BadPadding));
    assert_eq!(decode(b"MA===AAA"), Err(BadCharacter(5)));
    assert_eq!(decode(b"MAA====="), Err(BadCharacter(3)));
    assert_eq!(decode(b"MAABBB=="), Err(BadCharacter(6)));
}

test!{
    fn base32hex;
    test(b"", b"");
    test(b"f", b"CO======");
    test(b"fo", b"CPNG====");
    test(b"foo", b"CPNMU===");
    test(b"foob", b"CPNMUOG=");
    test(b"fooba", b"CPNMUOJ1");
    test(b"foobar", b"CPNMUOJ1E8======");
}

test!{
    fn base64;
    test(b"", b"");
    test(b"f", b"Zg==");
    test(b"fo", b"Zm8=");
    test(b"foo", b"Zm9v");
    test(b"foob", b"Zm9vYg==");
    test(b"fooba", b"Zm9vYmE=");
    test(b"foobar", b"Zm9vYmFy");
}

#[test]
fn exhaustive() {
    use data_encoding::base64::{encode_mut, decode_mut};
    let mut t = 0u32;
    let mut x = [0u8; 4];
    let mut y = [0u8; 3];
    let mut z = [0u8; 4];
    'iter: loop {
        if let Ok(n) = decode_mut(&x, &mut y) {
            t += 1;
            encode_mut(&y[0..n], &mut z);
            assert_eq!(x, z);
        }
        let mut i = 0;
        // We are only interested in all ascii and one non-ascii.
        while x[i] == 128 {
            x[i] = 0;
            i += 1;
            if i == 4 { break 'iter; }
        }
        x[i] += 1;
    }
    assert_eq!(t, 0x1010100_u32);
}
