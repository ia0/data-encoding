//! Macros for data-encoding
//!
//! This library provides macros to define compile-time byte arrays from encoded strings (using
//! common bases like [base64], [base32], or [hexadecimal], and also custom bases). It also provides
//! a macro to define compile-time custom encodings to be used with the [data-encoding] crate at
//! run-time.
//!
//! Up to Rust 1.50, you may need to add the following to your `.cargo/config.toml` to use this
//! library in no-std or no-alloc environments:
//!
//! ```toml
//! [unstable]
//! features = ["host_dep"]
//! ```
//!
//! From Rust 1.51, you may need to add the following to your `Cargo.toml`:
//!
//! ```toml
//! [package]
//! resolver = "2"
//! ```
//!
//! # Examples
//!
//! You can define a compile-time byte slice from an encoded string literal:
//!
//! ```rust
//! const HELLO_SLICE: &'static [u8] = &data_encoding_macro::hexlower!("68656c6c6f");
//! const FOOBAR_SLICE: &'static [u8] = &data_encoding_macro::base64!("Zm9vYmFy");
//! # fn main() {}
//! ```
//!
//! You can also define a compile-time byte array from an encoded string literal:
//!
//! ```rust
//! data_encoding_macro::hexlower_array!("const HELLO" = "68656c6c6f");
//! data_encoding_macro::base64_array!("const FOOBAR" = "Zm9vYmFy");
//! # fn main() {}
//! ```
//!
//! You can define a compile-time custom encoding from its specification:
//!
//! ```rust
//! const HEX: data_encoding::Encoding = data_encoding_macro::new_encoding! {
//!     symbols: "0123456789abcdef",
//!     translate_from: "ABCDEF",
//!     translate_to: "abcdef",
//! };
//! const BASE64: data_encoding::Encoding = data_encoding_macro::new_encoding! {
//!     symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
//!     padding: '=',
//! };
//! # fn main() {}
//! ```
//!
//! You can compute at compile-time the maximum length needed to decode inputs of a given length
//! (must be a integer literal). This is useful to allocate arrays on the stack:
//!
//! ```rust
//! let mut array = [0; data_encoding_macro::hexlower_decode_len!(26)];
//! assert_eq!(array.len(), 13);
//! let len = data_encoding::HEXLOWER.decode_mut(&[b'f'; 26], &mut array).unwrap();
//! assert_eq!(len, 13);
//! assert_eq!(array, [0xff; 13]);
//!
//! let mut array = [0; data_encoding_macro::base64_decode_len!(24)];
//! assert_eq!(array.len(), 18);
//! let len = data_encoding::BASE64.decode_mut(&[b'Q'; 24], &mut array).unwrap();
//! assert_eq!(len, 18);
//! assert_eq!(array, std::array::from_fn(|i| [0x41, 0x04, 0x10][i % 3]));
//! ```
//!
//! You can also compute at compile-time the length needed to encode inputs of a given length (must
//! be a integer literal). This is useful to allocate arrays on the stack:
//!
//! ```rust
//! let mut array = [0; data_encoding_macro::hexlower_encode_len!(13)];
//! assert_eq!(array.len(), 26);
//! data_encoding::HEXLOWER.encode_mut(&[0xff; 13], &mut array);
//! assert_eq!(array, [b'f'; 26]);
//!
//! let mut array = [0; data_encoding_macro::base64_encode_len!(18)];
//! assert_eq!(array.len(), 24);
//! let input: [u8; 18] = std::array::from_fn(|i| [0x41, 0x04, 0x10][i % 3]);
//! data_encoding::BASE64.encode_mut(&input, &mut array);
//! assert_eq!(array, [b'Q'; 24]);
//! ```
//!
//! [base32]: macro.base32.html
//! [base64]: macro.base64.html
//! [data-encoding]: https://crates.io/crates/data-encoding
//! [hexadecimal]: macro.hexlower_permissive.html

#![no_std]
#![warn(unused_results)]

#[doc(hidden)]
pub use data_encoding_macro_internal::{
    internal_decode_array, internal_decode_len, internal_decode_slice, internal_encode_len,
    internal_new_encoding,
};

/// Defines a compile-time byte array by decoding a string literal
///
/// This macro takes a list of `key: value,` pairs (the last comma is required). It takes the
/// key-value pairs specifying the encoding to use to decode the input (see [new_encoding] for the
/// possible key-value pairs), the input itself keyed by `input`, and the output keyed by `name`.
/// The output must be of the form `[pub] {const|static} <name>`.
///
/// # Examples
///
/// ```rust
/// data_encoding_macro::decode_array! {
///     name: "const OCTAL",
///     symbols: "01234567",
///     padding: '=',
///     input: "237610==",
/// }
/// # fn main() {}
/// ```
///
/// [new_encoding]: macro.new_encoding.html
#[macro_export]
macro_rules! decode_array {
    ($($arg: tt)*) => {
        $crate::internal_decode_array!($($arg)*);
    };
}

/// Defines a compile-time byte slice by decoding a string literal
///
/// This macro takes a list of `key: value,` pairs (the last comma is required). It takes the
/// key-value pairs specifying the encoding to use to decode the input (see [new_encoding] for the
/// possible key-value pairs) and the input itself keyed by `input`.
///
/// # Examples
///
/// ```rust
/// const OCTAL: &'static [u8] = &data_encoding_macro::decode_slice! {
///     symbols: "01234567",
///     padding: '=',
///     input: "237610==",
/// };
/// # fn main() {}
/// ```
///
/// [new_encoding]: macro.new_encoding.html
#[macro_export]
macro_rules! decode_slice {
    ($($arg: tt)*) => {
        $crate::internal_decode_slice!($($arg)*)
    };
}

/// Defines a compile-time custom encoding
///
/// This macro takes a list of `key: value,` pairs (the last comma is required). The possible
/// key-value pairs are:
///
/// ```text
///             symbols: <string>,       // e.g. "01234567"
///             padding: [None]|<char>,  // e.g. '='
///           bit_order: [MostSignificantFirst]|LeastSignificantFirst,
/// check_trailing_bits: [true]|false,
///              ignore: [""]|<string>,  // e.g. " \t\n"
///          wrap_width: [0]|<int>,      // e.g. 76
///      wrap_separator: [""]|<string>,  // e.g. "\r\n"
///      translate_from: [""]|<string>,  // e.g. "ABCDEF"
///        translate_to: [""]|<string>,  // e.g. "abcdef"
/// ```
///
/// Only `symbols` is required. Everything else is optional and defaults to the value between square
/// brackets.
///
/// # Examples
///
/// ```rust
/// const HEX: data_encoding::Encoding = data_encoding_macro::new_encoding! {
///     symbols: "0123456789abcdef",
///     ignore: " \r\t\n",
///     wrap_width: 32,
///     wrap_separator: "\n",
///     translate_from: "ABCDEF",
///     translate_to: "abcdef",
/// };
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! new_encoding {
    ($($arg: tt)*) => {
        data_encoding::Encoding::internal_new(&$crate::internal_new_encoding!{ $($arg)* })
    };
}

/// Computes at compile-time the maximum length needed to decode inputs of a given length
///
/// This macro takes a list of `key: value,` pairs (the last comma is required). It takes the
/// key-value pairs specifying the encoding to use to decode the input (see [new_encoding] for the
/// possible key-value pairs) and the input itself keyed by `input`.
///
/// # Examples
///
/// ```rust
/// data_encoding_macro::decode_len! {
///     symbols: "01234567",
///     padding: '=',
///     input: 32,
/// }
/// ```
///
/// [new_encoding]: macro.new_encoding.html
#[macro_export]
macro_rules! decode_len {
    ($($arg: tt)*) => {
        $crate::internal_decode_len!($($arg)*);
    };
}

/// Computes at compile-time the length needed to encode inputs of a given length
///
/// This macro takes a list of `key: value,` pairs (the last comma is required). It takes the
/// key-value pairs specifying the encoding to use to decode the input (see [new_encoding] for the
/// possible key-value pairs) and the input itself keyed by `input`.
///
/// # Examples
///
/// ```rust
/// data_encoding_macro::encode_len! {
///     symbols: "01234567",
///     padding: '=',
///     input: 20,
/// }
/// ```
///
/// [new_encoding]: macro.new_encoding.html
#[macro_export]
macro_rules! encode_len {
    ($($arg: tt)*) => {
        $crate::internal_encode_len!($($arg)*);
    };
}

macro_rules! make {
    ($base: ident $base_array: ident $base_encode_len: ident $base_decode_len: ident = $ref: ident;
     $($spec: tt)*) => {
        #[macro_export]
        macro_rules! $base_array {
            ($n: tt = $x: tt) => {
                $crate::decode_array!(name: $n, input: $x, $($spec)*);
            };
        }
        #[macro_export]
        macro_rules! $base {
            ($x: tt) => {
                $crate::decode_slice!(input: $x, $($spec)*)
            };
        }
        #[macro_export]
        macro_rules! $base_encode_len {
            ($x: tt) => {
                $crate::encode_len!(input: $x, $($spec)*)
            };
        }
        #[macro_export]
        macro_rules! $base_decode_len {
            ($x: tt) => {
                $crate::decode_len!(input: $x, $($spec)*)
            };
        }
        #[test]
        fn $base() {
            assert_eq!(new_encoding!($($spec)*), data_encoding::$ref);
        }
    };
}

make! {
    hexlower hexlower_array hexlower_encode_len hexlower_decode_len = HEXLOWER;
    symbols: "0123456789abcdef",
}
make! {
    hexlower_permissive hexlower_permissive_array hexlower_permissive_encode_len
        hexlower_permissive_decode_len = HEXLOWER_PERMISSIVE;
    symbols: "0123456789abcdef",
    translate_from: "ABCDEF",
    translate_to: "abcdef",
}
make! {
    hexupper hexupper_array hexupper_encode_len hexupper_decode_len = HEXUPPER;
    symbols: "0123456789ABCDEF",
}
make! {
    hexupper_permissive hexupper_permissive_array hexupper_permissive_encode_len
        hexupper_permissive_decode_len = HEXUPPER_PERMISSIVE;
    symbols: "0123456789ABCDEF",
    translate_from: "abcdef",
    translate_to: "ABCDEF",
}
make! {
    base32 base32_array base32_encode_len base32_decode_len = BASE32;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
    padding: '=',
}
make! {
    base32_nopad base32_nopad_array base32_nopad_encode_len base32_nopad_decode_len = BASE32_NOPAD;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
}
make! {
    base32hex base32hex_array base32hex_encode_len base32hex_decode_len = BASE32HEX;
    symbols: "0123456789ABCDEFGHIJKLMNOPQRSTUV",
    padding: '=',
}
make! {
    base32hex_nopad base32hex_nopad_array base32hex_nopad_encode_len base32hex_nopad_decode_len =
        BASE32HEX_NOPAD;
    symbols: "0123456789ABCDEFGHIJKLMNOPQRSTUV",
}
make! {
    base32_dnssec base32_dnssec_array base32_dnssec_encode_len base32_dnssec_decode_len =
        BASE32_DNSSEC;
    symbols: "0123456789abcdefghijklmnopqrstuv",
    translate_from: "ABCDEFGHIJKLMNOPQRSTUV",
    translate_to: "abcdefghijklmnopqrstuv",
}
make! {
    base32_dnscurve base32_dnscurve_array base32_dnscurve_encode_len base32_dnscurve_decode_len =
        BASE32_DNSCURVE;
    symbols: "0123456789bcdfghjklmnpqrstuvwxyz",
    bit_order: LeastSignificantFirst,
    translate_from: "BCDFGHJKLMNPQRSTUVWXYZ",
    translate_to: "bcdfghjklmnpqrstuvwxyz",
}
make! {
    base64 base64_array base64_encode_len base64_decode_len = BASE64;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    padding: '=',
}
make! {
    base64_nopad base64_nopad_array base64_nopad_encode_len base64_nopad_decode_len = BASE64_NOPAD;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
}
make! {
    base64_mime base64_mime_array base64_mime_encode_len base64_mime_decode_len = BASE64_MIME;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    padding: '=',
    wrap_width: 76,
    wrap_separator: "\r\n",
}
make! {
    base64url base64url_array base64url_encode_len base64url_decode_len = BASE64URL;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
    padding: '=',
}
make! {
    base64url_nopad base64url_nopad_array base64url_nopad_encode_len base64url_nopad_decode_len =
        BASE64URL_NOPAD;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
}
