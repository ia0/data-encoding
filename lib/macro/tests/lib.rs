#![cfg_attr(not(feature = "stable"), feature(use_extern_macros))]

extern crate data_encoding;
#[macro_use]
extern crate data_encoding_macro;

use std::ops::Deref;

use data_encoding::{BASE64, HEXLOWER};

// Test the macro invocation from inside a module.
mod test {
    const BASE32_DNSCURVE: ::data_encoding::Encoding = new_encoding!{
        symbols: "0123456789bcdfghjklmnpqrstuvwxyz",
        bit_order: LeastSignificantFirst,
        translate_from: "BCDFGHJKLMNPQRSTUVWXYZ",
        translate_to: "bcdfghjklmnpqrstuvwxyz",
    };

    #[test]
    fn base32_dnscurve() {
        assert_eq!(BASE32_DNSCURVE, ::data_encoding::BASE32_DNSCURVE);
    }
}

#[cfg(not(feature = "stable"))]
#[test]
fn decode_array() {
    macro_rules! base {
        ($f: ident; $($x: tt)*) => {
            $f!{
                symbols: "0123456789abcdef",
                bit_order: LeastSignificantFirst,
                padding: None,
                $($x)*
            }
        };
    }
    base!{decode_array;
        name: "const OUTPUT",
        input: "deadbeef",
    }
    const BASE: ::data_encoding::Encoding = base!(new_encoding;);
    assert_eq!(&OUTPUT, BASE.decode(b"deadbeef").unwrap().deref());
}

#[test]
fn decode_slice() {
    macro_rules! base {
        ($f: ident; $($x: tt)*) => {
            $f!{
                symbols: "0123456789abcdef",
                bit_order: LeastSignificantFirst,
                padding: None,
                $($x)*
            }
        };
    }
    const OUTPUT: &'static [u8] = &base!{
        decode_slice;
        input: "deadbeef",
    };
    const BASE: ::data_encoding::Encoding = base!(new_encoding;);
    assert_eq!(OUTPUT, BASE.decode(b"deadbeef").unwrap().deref());
}

#[cfg(not(feature = "stable"))]
#[test]
fn hexlower_decode_array() {
    hexlower_array!("const OUTPUT" = "deadbeef");
    assert_eq!(&OUTPUT, HEXLOWER.decode(b"deadbeef").unwrap().deref());
}

#[cfg(not(feature = "stable"))]
#[test]
fn base64_decode_array() {
    base64_array!("const OUTPUT" = "deadbeef");
    assert_eq!(&OUTPUT, BASE64.decode(b"deadbeef").unwrap().deref());
}

#[test]
fn hexlower_decode() {
    const OUTPUT: &'static [u8] = &hexlower!("deadbeef");
    assert_eq!(OUTPUT, HEXLOWER.decode(b"deadbeef").unwrap().deref());
}

#[test]
fn base64_decode() {
    const OUTPUT: &'static [u8] = &base64!("deadbeef");
    assert_eq!(OUTPUT, BASE64.decode(b"deadbeef").unwrap().deref());
}
