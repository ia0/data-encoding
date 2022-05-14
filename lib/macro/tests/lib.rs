// Test the macro invocation from inside a module.
mod test {
    const BASE32_DNSCURVE: data_encoding::Encoding = data_encoding_macro::new_encoding! {
        symbols: "0123456789bcdfghjklmnpqrstuvwxyz",
        bit_order: LeastSignificantFirst,
        translate_from: "BCDFGHJKLMNPQRSTUVWXYZ",
        translate_to: "bcdfghjklmnpqrstuvwxyz",
    };

    #[test]
    fn base32_dnscurve() {
        assert_eq!(BASE32_DNSCURVE, data_encoding::BASE32_DNSCURVE);
    }
}

#[test]
fn decode_array() {
    macro_rules! base {
        ($f: ident; $($x: tt)*) => {
            data_encoding_macro::$f!{
                symbols: "0123456789abcdef",
                bit_order: LeastSignificantFirst,
                padding: None,
                $($x)*
            }
        };
    }
    base! {decode_array;
        name: "const OUTPUT",
        input: "deadbeef",
    }
    const BASE: data_encoding::Encoding = base!(new_encoding;);
    assert_eq!(&OUTPUT as &[u8], BASE.decode(b"deadbeef").unwrap());
}

#[test]
fn decode_slice() {
    macro_rules! base {
        ($f: ident; $($x: tt)*) => {
            data_encoding_macro::$f!{
                symbols: "0123456789abcdef",
                bit_order: LeastSignificantFirst,
                padding: None,
                $($x)*
            }
        };
    }
    const OUTPUT: &'static [u8] = &base! {
        decode_slice;
        input: "deadbeef",
    };
    const BASE: data_encoding::Encoding = base!(new_encoding;);
    assert_eq!(OUTPUT, BASE.decode(b"deadbeef").unwrap());
}

#[test]
fn hexlower_decode_array() {
    data_encoding_macro::hexlower_array!("const OUTPUT" = "deadbeef");
    assert_eq!(&OUTPUT as &[u8], data_encoding::HEXLOWER.decode(b"deadbeef").unwrap());
}

#[test]
fn base64_decode_array() {
    data_encoding_macro::base64_array!("const OUTPUT" = "deadbeef");
    assert_eq!(&OUTPUT as &[u8], data_encoding::BASE64.decode(b"deadbeef").unwrap());
}

#[test]
fn hexlower_decode() {
    const OUTPUT: &'static [u8] = &data_encoding_macro::hexlower!("deadbeef");
    assert_eq!(OUTPUT, data_encoding::HEXLOWER.decode(b"deadbeef").unwrap());
}

#[test]
fn base64_decode() {
    const OUTPUT: &'static [u8] = &data_encoding_macro::base64!("deadbeef");
    assert_eq!(OUTPUT, data_encoding::BASE64.decode(b"deadbeef").unwrap());
}

#[test]
fn escaped_symbols() {
    const BASE: data_encoding::Encoding = data_encoding_macro::new_encoding! {
        symbols: "\x00\n\"\\",
    };
    assert_eq!(BASE.encode(b"K"), "\n\x00\"\\");
}
