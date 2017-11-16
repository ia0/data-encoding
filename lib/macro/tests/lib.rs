#![feature(use_extern_macros)]

extern crate data_encoding;
#[macro_use]
extern crate data_encoding_macro;

use std::ops::Deref;

use data_encoding::{BASE64, HEXLOWER};
use data_encoding_macro::decode;

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

#[test]
fn decode() {
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
    base!{decode;
        name: "const OUTPUT",
        input: "deadbeef",
    }
    const BASE: ::data_encoding::Encoding = base!(new_encoding;);
    assert_eq!(&OUTPUT, BASE.decode(b"deadbeef").unwrap().deref());
}

#[test]
fn hexlower_decode() {
    hexlower!("const OUTPUT" = "deadbeef");
    assert_eq!(&OUTPUT, HEXLOWER.decode(b"deadbeef").unwrap().deref());
}

#[test]
fn base64_decode() {
    base64!("const OUTPUT" = "deadbeef");
    assert_eq!(&OUTPUT, BASE64.decode(b"deadbeef").unwrap().deref());
}
