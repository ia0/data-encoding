//! This crate provides generic data encoding functions.
//!
//! Encoding and decoding functions are provided for common bases.
//! Those functions are instantiated from generic functions using a
//! base interface described in module [`base`](base/index.html). The
//! generic encoding and decoding functions are defined in the
//! [`encode`](encode/index.html) and [`decode`](decode/index.html)
//! modules respectively.
//!
//! # Examples
//!
//! ```
//! use data_encoding::hex;
//! use data_encoding::base64;
//! assert_eq!(hex::encode(b"some raw data").as_bytes(), b"736F6D65207261772064617461");
//! assert_eq!(base64::decode(b"c29tZSByYXcgZGF0YQ==").unwrap(), b"some raw data");
//! ```
//!
//! A more involved example is available in the `examples` directory.
//!
//! # Properties
//!
//! The encoding and decoding functions satisfy the following
//! properties:
//!
//! - They are deterministic: their output only depends on their input.
//! - They have no side-effects: they do not modify a hidden mutable
//! state.
//! - They never panic, although the decoding function may return a
//! decoding error on invalid input.
//! - They are inverses of each other:
//!   - For all `data: Vec<u8>`, we have
//!   `decode(encode(&data).as_bytes()) == Ok(data)`.
//!   - For all `repr: String`, if there is `data: Vec<u8>` such that
//!   `decode(repr.as_bytes()) == Ok(data)`, then `encode(&data) ==
//!   repr`.
//!
//! # Conformance
//!
//! The [`base16`](base16/index.html), [`hex`](index.html#reexports),
//! [`base32`](base32/index.html),
//! [`base32hex`](base32hex/index.html),
//! [`base64`](base64/index.html), and
//! [`base64url`](base64url/index.html) modules are conform to [RFC
//! 4648](https://tools.ietf.org/html/rfc4648).
//!
//! # Performance
//!
//! This [crate](https://crates.io/crates/data-encoding) has
//! comparable performance to the `rustc-serialize` crate and the
//! `base64` GNU program.

#![warn(unused_results)]

#[macro_use]
mod tool;
pub mod base;
pub mod encode;
pub mod decode;

// Rust is missing functors: I use macros.

macro_rules! ascii {
    ($($v: expr),*) => { &[
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        $($v),*,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
        ] };
}
macro_rules! base {
    (#[$d: meta] $(#[$a: meta])* mod $n: ident;
     $b: expr, $p: expr, $r: expr, $s: expr, $($v: expr),*,) =>
    {
        #[$d]
        ///
        /// See the generic [`encode`](../encode/index.html) and
        /// [`decode`](../decode/index.html) modules for details about
        /// this module functions.
        ///
        /// # Definition
        $(#[$a])*
        pub mod $n {
            use ::decode::Error;
            const X_: u8 = 128;
            enum Static {}
            static BASE: ::base::Opt<Static> = ::base::Opt {
                val: ascii!($($v),*), sym: $s, bit: $b, pad: $p,
                _phantom: ::std::marker::PhantomData
            };
            pub fn encode_len(len: usize) -> usize {
                ::encode::encode_len(&BASE, len)
            }
            pub fn decode_len(len: usize) -> usize {
                ::decode::decode_len(&BASE, len)
            }
            pub fn encode_mut(input: &[u8], output: &mut [u8]) {
                ::encode::encode_mut(&BASE, input, output)
            }
            pub fn decode_mut(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
                ::decode::decode_mut(&BASE, input, output)
            }
            pub fn encode(input: &[u8]) -> String {
                ::encode::encode(&BASE, input)
            }
            pub fn decode(input: &[u8]) -> Result<Vec<u8>, Error> {
                ::decode::decode(&BASE, input)
            }
            #[test]
            fn check() {
                use base::{Spec, equal, valid};
                const SPEC: Spec = Spec { val: $r, pad: $p };
                assert_eq!(BASE.val.len(), 256);
                assert_eq!(BASE.sym.len(), 1 << BASE.bit);
                equal(&BASE, &SPEC).unwrap();
                valid(&BASE).unwrap();
            }
        }
    };
}

base!{
    /// Base 2 Encoding.
    ///
    /// Symbols are `01`. No padding is required.
    mod base2;
    1, b'=', &[(b'0', b'1')], b"01",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0_, 1_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
}

base!{
    /// Base 4 Encoding.
    ///
    /// Symbols are `0-3`. No padding is required.
    mod base4;
    2, b'=', &[(b'0', b'3')], b"0123",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0_, 1_, 2_, 3_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
}

base!{
    /// Base 8 Encoding.
    ///
    /// Symbols are `0-7`. Padding is `=`.
    mod base8;
    3, b'=', &[(b'0', b'7')], b"01234567",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
}

base!{
    /// Base 16 Encoding.
    ///
    /// Symbols are `0-9A-F`. No padding is required.
    ///
    /// # Conformance
    ///
    /// [RFC 4648](https://tools.ietf.org/html/rfc4648#section-8) compliant.
    mod base16;
    4, b'=', &[(b'0', b'9'), (b'A', b'F')], b"0123456789ABCDEF",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, X_, X_, X_, X_, X_, X_,
    X_, 10, 11, 12, 13, 14, 15, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
}
pub use base16 as hex;

base!{
    /// Base 32 Encoding.
    ///
    /// Symbols are `A-Z2-7`. Padding is `=`.
    ///
    /// # Conformance
    ///
    /// [RFC 4648](https://tools.ietf.org/html/rfc4648#section-6) compliant.
    mod base32;
    5, b'=', &[(b'A', b'Z'), (b'2', b'7')],
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, 26, 27, 28, 29, 30, 31, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, 0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, 10, 11, 12, 13, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
}

base!{
    /// Base 32 Encoding with Extended Hex Alphabet.
    ///
    /// Symbols are `0-9A-V`. Padding is `=`.
    ///
    /// # Conformance
    ///
    /// [RFC 4648](https://tools.ietf.org/html/rfc4648#section-7) compliant.
    mod base32hex;
    5, b'=', &[(b'0', b'9'), (b'A', b'V')],
    b"0123456789ABCDEFGHIJKLMNOPQRSTUV",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, X_, X_, X_, X_, X_, X_,
    X_, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 26, 27, 28, 29, 30, 31, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
}

base!{
    /// Base 64 Encoding.
    ///
    /// Symbols are `A-Za-z0-9+/`. Padding is `=`.
    ///
    /// # Conformance
    ///
    /// [RFC 4648](https://tools.ietf.org/html/rfc4648#section-4) compliant.
    mod base64;
    6, b'=', &[(b'A', b'Z'), (b'a', b'z'), (b'0', b'9'), (b'+', b'+'), (b'/', b'/')],
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, 62, X_, X_, X_, 63,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, X_, X_, X_, X_, X_, X_,
    X_, 0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, 10, 11, 12, 13, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, X_, X_, X_, X_, X_,
    X_, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, X_, X_, X_, X_, X_,
}

base!{
    /// Base 64 Encoding with URL and Filename Safe Alphabet.
    ///
    /// Symbols are `A-Za-z0-9-_`. Padding is `=`.
    ///
    /// # Conformance
    ///
    /// [RFC 4648](https://tools.ietf.org/html/rfc4648#section-5) compliant.
    mod base64url;
    6, b'=', &[(b'A', b'Z'), (b'a', b'z'), (b'0', b'9'), (b'-', b'-'), (b'_', b'_')],
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, 62, X_, X_,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, X_, X_, X_, X_, X_, X_,
    X_, 0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, 10, 11, 12, 13, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, X_, X_, X_, X_, 63,
    X_, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, X_, X_, X_, X_, X_,
}
