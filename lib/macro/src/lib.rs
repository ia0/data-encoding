//! Macros for data-encoding
//!
//! This library requires a nightly compiler because it uses procedural macros.
//!
//! It provides macros to define compile-time byte arrays from encoded strings
//! (using common bases like [base64], [base32], or [hexadecimal], and also
//! custom bases). It also provides a macro to define compile-time custom
//! encodings to be used with the [data-encoding] crate at run-time.
//!
//! If you were familiar with the [binary_macros] crate, this library is
//! actually [inspired][binary_macros_issue] from it.
//!
//! # Examples
//!
//! You can define a compile-time byte array from an encoded string literal:
//!
//! ```rust
//! #![feature(use_extern_macros)]
//! #[macro_use] extern crate data_encoding_macro;
//!
//! hexlower!("const HELLO" = "68656c6c6f");
//! base64!("const FOOBAR" = "Zm9vYmFy");
//! # fn main() {}
//! ```
//!
//! You can define a compile-time custom encoding from its specification:
//!
//! ```rust
//! #![feature(use_extern_macros)]
//! extern crate data_encoding;
//! #[macro_use] extern crate data_encoding_macro;
//! use data_encoding::Encoding;
//!
//! const HEX: Encoding = new_encoding!{
//!     symbols: "0123456789abcdef",
//!     translate_from: "ABCDEF",
//!     translate_to: "abcdef",
//! };
//! const BASE64: Encoding = new_encoding!{
//!     symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
//!     padding: '=',
//! };
//! # fn main() {}
//! ```
//!
//! [base32]: macro.base32.html
//! [base64]: macro.base64.html
//! [binary_macros]: https://crates.io/crates/binary_macros
//! [binary_macros_issue]: https://github.com/ia0/data-encoding/issues/7
//! [data-encoding]: https://crates.io/crates/data-encoding
//! [hexadecimal]: macro.hexlower_permissive.html

#![feature(use_extern_macros)]
#![warn(unused_results)]

extern crate data_encoding;
extern crate data_encoding_macro_internal;

#[doc(hidden)]
pub use data_encoding_macro_internal::internal_new_encoding;
#[doc(hidden)]
pub use data_encoding_macro_internal::internal_decode;

/// Defines a compile-time byte array by decoding a string literal
///
/// This macro takes a list of `key: value,` pairs (the last comma is required).
/// It takes the key-value pairs specifying the encoding to use to decode the
/// input (see [new_encoding] for the possible key-value pairs), the input
/// itself keyed by `input`, and the output keyed by `name`. The output must be
/// of the form `[pub] {const|static} <name>`.
///
/// # Examples
///
/// ```rust
/// #![feature(use_extern_macros)]
/// #[macro_use] extern crate data_encoding_macro;
///
/// decode!{
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
macro_rules! decode {
    ($($arg: tt)*) => {
        $crate::internal_decode!($($arg)*);
    };
}

/// Defines a compile-time custom encoding
///
/// This macro takes a list of `key: value,` pairs (the last comma is required).
/// The possible key-value pairs are:
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
/// Only `symbols` is required. Everything else is optional and defaults to the
/// value between square brackets.
///
/// # Examples
///
/// ```rust
/// #![feature(use_extern_macros)]
/// extern crate data_encoding;
/// #[macro_use] extern crate data_encoding_macro;
///
/// const HEX: data_encoding::Encoding = new_encoding!{
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
        ::data_encoding::Encoding(::std::borrow::Cow::Borrowed(
            &$crate::internal_new_encoding!{ $($arg)* }))
    };
}

macro_rules! make {
    ($base: ident = $ref: ident; $($spec: tt)*) => {
        #[macro_export]
        macro_rules! $base {
            ($n: tt = $x: tt) => {
                decode!(name: $n, input: $x, $($spec)*);
            };
        }
        #[test]
        fn $base() {
            assert_eq!(new_encoding!($($spec)*), data_encoding::$ref);
        }
    };
}

make!{
    hexlower = HEXLOWER;
    symbols: "0123456789abcdef",
}
make!{
    hexlower_permissive = HEXLOWER_PERMISSIVE;
    symbols: "0123456789abcdef",
    translate_from: "ABCDEF",
    translate_to: "abcdef",
}
make!{
    hexupper = HEXUPPER;
    symbols: "0123456789ABCDEF",
}
make!{
    hexupper_permissive = HEXUPPER_PERMISSIVE;
    symbols: "0123456789ABCDEF",
    translate_from: "abcdef",
    translate_to: "ABCDEF",
}
make!{
    base32 = BASE32;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
    padding: '=',
}
make!{
    base32_nopad = BASE32_NOPAD;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
}
make!{
    base32hex = BASE32HEX;
    symbols: "0123456789ABCDEFGHIJKLMNOPQRSTUV",
    padding: '=',
}
make!{
    base32hex_nopad = BASE32HEX_NOPAD;
    symbols: "0123456789ABCDEFGHIJKLMNOPQRSTUV",
}
make!{
    base32_dnssec = BASE32_DNSSEC;
    symbols: "0123456789abcdefghijklmnopqrstuv",
    translate_from: "ABCDEFGHIJKLMNOPQRSTUV",
    translate_to: "abcdefghijklmnopqrstuv",
}
make!{
    base32_dnscurve = BASE32_DNSCURVE;
    symbols: "0123456789bcdfghjklmnpqrstuvwxyz",
    bit_order: LeastSignificantFirst,
    translate_from: "BCDFGHJKLMNPQRSTUVWXYZ",
    translate_to: "bcdfghjklmnpqrstuvwxyz",
}
make!{
    base64 = BASE64;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    padding: '=',
}
make!{
    base64_nopad = BASE64_NOPAD;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
}
make!{
    base64_mime = BASE64_MIME;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    padding: '=',
    wrap_width: 76,
    wrap_separator: "\r\n",
}
make!{
    base64url = BASE64URL;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
    padding: '=',
}
make!{
    base64url_nopad = BASE64URL_NOPAD;
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
}
