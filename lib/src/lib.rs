//! Correct, efficient, canonical, and generic data-encoding functions
//!
//! This [crate] provides little-endian ASCII base-conversion encodings for
//! bases of size 2, 4, 8, 16, 32, and 64. It supports both padded and
//! non-padded encodings. It supports canonical encodings (trailing bits are
//! checked). It supports in-place encoding and decoding functions. It supports
//! non-canonical symbols. And it supports both most and least significant
//! bit-order. The performance of the encoding and decoding functions are
//! similar to existing implementations (see how to run the benchmarks on
//! [github]).
//!
//! This is the library documentation. If you are looking for the [binary], see
//! the installation instructions on [github].
//!
//! # Examples
//!
//! This crate provides predefined encodings as [constants]. These constants are
//! of type [`Padded`] or [`NoPad`] whether they use padding or not. These types
//! provide encoding and decoding functions with in-place or allocating
//! variants. Here is an example using the allocating encoding function of
//! [base64]:
//!
//! ```
//! use data_encoding::BASE64;
//! assert_eq!(BASE64.encode(b"Hello world"), "SGVsbG8gd29ybGQ=");
//! ```
//!
//! It is also possible to use the non-padded version of base64 by calling the
//! [`no_pad`] method of [`Padded`]:
//!
//! ```
//! use data_encoding::BASE64;
//! assert_eq!(BASE64.no_pad().encode(b"Hello world"), "SGVsbG8gd29ybGQ");
//! ```
//!
//! Here is an example using the in-place decoding function of [base32]:
//!
//! ```
//! use data_encoding::BASE32;
//! let input = b"JBSWY3DPEB3W64TMMQ======";
//! let mut output = vec![0; BASE32.decode_len(input.len()).unwrap()];
//! let len = BASE32.decode_mut(input, &mut output).unwrap();
//! assert_eq!(&output[0 .. len], b"Hello world");
//! ```
//!
//! You are not limited to the predefined encodings. You may define your own
//! encodings (with the same correctness and performance properties as the
//! predefined ones) using the [`Builder`] type:
//!
//! ```rust
//! use data_encoding::Builder;
//! let hex = Builder::new(b"0123456789abcdef").no_pad().unwrap();
//! assert_eq!(hex.encode(b"hello"), "68656c6c6f");
//! ```
//!
//! If you use the `lazy_static` crate, you can define a global base:
//!
//! ```rust,ignore
//! lazy_static! {
//!     static ref BASE: NoPad = Builder::new(b"0123456789abcdef")
//!         .translate(b"ABCDEF", b"abcdef").no_pad().unwrap();
//! }
//! ```
//!
//! # Properties
//!
//! The [base16], [base32], [base32hex], [base64], and [base64url] predefined
//! encodings are conform to [RFC4648].
//!
//! The encoding and decoding functions satisfy the following properties:
//!
//! - They are deterministic: their output only depends on their input
//! - They have no side-effects: they do not modify a hidden mutable state
//! - They are correct: encoding then decoding gives the initial data
//! - They are canonical (unless non-canonical symbols are used or checking
//! trailing bits is disabled): decoding then encoding gives the initial data
//!
//! This last property is usually not satisfied by common base64 implementations
//! (like the `rustc-serialize` crate, the `base64` crate, or the `base64` GNU
//! program). This is a matter of choice and this crate has made the choice to
//! let the user choose. Support for canonical encoding as described by the
//! [RFC][canonical] is provided. But it is also possible to disable checking
//! trailing bits, to add non-canonical symbols, and to decode concatenated
//! padded inputs.
//!
//! Since the RFC specifies the encoding function on all inputs and the decoding
//! function on all possible encoded outputs, the differences between
//! implementations come from the decoding function which may be more or less
//! permissive. In this crate, the decoding function of canonical encodings
//! rejects all inputs that are not a possible output of the encoding function.
//! Here are some concrete examples of decoding differences between this crate,
//! the `rustc-serialize` crate, the `base64` crate, and the `base64` GNU
//! program:
//!
//! | Input      | `data-encoding` | `rustc`  | `base64` | GNU `base64`  |
//! | ---------- | --------------- | -------- | -------- | ------------- |
//! | `AAB=`     | `Trailing(2)`   | `[0, 0]` | `[0, 0]` | `\x00\x00`    |
//! | `AA\nB=`   | `Length(4)`     | `[0, 0]` | `Err(2)` | `\x00\x00`    |
//! | `AAB`      | `Length(0)`     | `[0, 0]` | `[0, 0]` | Invalid input |
//! | `A\rA\nB=` | `Length(4)`     | `[0, 0]` | `Err(1)` | Invalid input |
//! | `-_\r\n`   | `Symbol(0)`     | `[251]`  | `Err(0)` | Invalid input |
//! | `AA==AA==` | `Symbol(2)`     | `Err`    | `Err(2)` | `\x00\x00`    |
//!
//! We can summarize these discrepancies as follows:
//!
//! | Discrepancy | `data-encoding` | `rustc` | `base64` | GNU `base64` |
//! | ----------- | --------------- | ------- | -------- | ------------ |
//! | Non-zero trailing bits | No | Yes | Yes | Yes |
//! | Ignored characters | None | `\r` and `\n` | None | `\n` |
//! | Translated characters | None | `-_` mapped to `+/` | None | None |
//! | Padding is optional | No | Yes | Yes | No |
//! | Concatenated padded input | No | No | No | Yes |
//!
//! This crate permits to [ignore][trailing] non-zero trailing bits. It permits
//! to [translate] symbols. It permits to use [non-padded][`NoPad`] encodings.
//! And it also permits to [decode][decode_concat] concatenated padded inputs.
//! However, it does not permit to ignore characters. This has to be done in a
//! preprocessing stage, as it is done in the [binary]. Support in the library
//! may be added in future versions.
//!
//! # Migration
//!
//! The [changelog] describes the changes between v1 and v2. Here are the
//! migration steps for common usage:
//!
//! | v1                          | v2                          |
//! | --------------------------- | --------------------------- |
//! | `use data_encoding::baseNN` | `use data_encoding::BASENN` |
//! | `baseNN::function`          | `BASENN.method`             |
//! | `baseNN::function_nopad`    | `BASENN.no_pad().method`    |
//!
//! [`Builder`]: struct.Builder.html
//! [`NoPad`]: struct.NoPad.html
//! [`Padded`]: struct.Padded.html
//! [RFC4648]: https://tools.ietf.org/html/rfc4648
//! [base16]: constant.HEXUPPER.html
//! [base32]: constant.BASE32.html
//! [base32hex]: constant.BASE32HEX.html
//! [base64]: constant.BASE64.html
//! [base64url]: constant.BASE64URL.html
//! [binary]: https://crates.io/crates/data-encoding-bin
//! [canonical]: https://tools.ietf.org/html/rfc4648#section-3.5
//! [changelog]: https://github.com/ia0/data-encoding/blob/master/lib/CHANGELOG.md
//! [constants]: index.html#constants
//! [crate]: https://crates.io/crates/data-encoding
//! [decode_concat]: struct.Padded.html#method.decode_concat
//! [github]: https://github.com/ia0/data-encoding
//! [`no_pad`]: struct.Padded.html#method.no_pad
//! [trailing]: struct.Builder.html#method.ignore_trailing_bits
//! [translate]: struct.Builder.html#method.translate

#![warn(unused_results, missing_docs)]

/// Decoding error kind
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DecodeKind {
    /// Invalid length
    Length,

    /// Invalid symbol
    Symbol,

    /// Non-zero trailing bits
    Trailing,

    /// Invalid padding length
    Padding,
}

/// Decoding error
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DecodeError {
    /// Error position
    pub position: usize,

    /// Error kind
    pub kind: DecodeKind,
}
impl std::error::Error for DecodeError {
    fn description(&self) -> &str {
        match self.kind {
            DecodeKind::Length => "invalid length",
            DecodeKind::Symbol => "invalid symbol",
            DecodeKind::Trailing => "non-zero trailing bits",
            DecodeKind::Padding => "invalid padding length",
        }
    }
}
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "{} at {}", self.description(), self.position)
    }
}

macro_rules! check { ($e: expr, $c: expr) => { if !$c { return Err($e); } }; }

fn div_ceil(x: usize, m: usize) -> usize { (x + m - 1) / m }
fn floor(x: usize, m: usize) -> usize { x / m * m }

unsafe fn chunk_unchecked(x: &[u8], n: usize, i: usize) -> &[u8] {
    debug_assert!((i + 1) * n <= x.len());
    let ptr = x.as_ptr().offset((n * i) as isize);
    std::slice::from_raw_parts(ptr, n)
}
unsafe fn chunk_mut_unchecked(x: &mut [u8], n: usize, i: usize) -> &mut [u8] {
    debug_assert!((i + 1) * n <= x.len());
    let ptr = x.as_mut_ptr().offset((n * i) as isize);
    std::slice::from_raw_parts_mut(ptr, n)
}

trait Base: Copy {
    fn bit(&self) -> usize;
    fn msb(&self) -> bool;
}

macro_rules! make {
    ($val: expr, $msb: ident, $lsb: ident) => {
        #[derive(Copy, Clone)] struct $msb;
        impl Base for $msb {
            fn bit(&self) -> usize { $val }
            fn msb(&self) -> bool { true }
        }
        #[derive(Copy, Clone)] struct $lsb;
        impl Base for $lsb {
            fn bit(&self) -> usize { $val }
            fn msb(&self) -> bool { false }
        }
    };
}
make!(1, M1, L1);
make!(2, M2, L2);
make!(3, M3, L3);
make!(4, M4, L4);
make!(5, M5, L5);
make!(6, M6, L6);

macro_rules! dispatch {
    ($fun: ident; $bit: expr, $msb: expr, $($arg: expr),*) => {
        match ($bit, $msb) {
            (1, true) => $fun(M1, $($arg),*),
            (2, true) => $fun(M2, $($arg),*),
            (3, true) => $fun(M3, $($arg),*),
            (4, true) => $fun(M4, $($arg),*),
            (5, true) => $fun(M5, $($arg),*),
            (6, true) => $fun(M6, $($arg),*),
            (1, false) => $fun(L1, $($arg),*),
            (2, false) => $fun(L2, $($arg),*),
            (3, false) => $fun(L3, $($arg),*),
            (4, false) => $fun(L4, $($arg),*),
            (5, false) => $fun(L5, $($arg),*),
            (6, false) => $fun(L6, $($arg),*),
            _ => unreachable!(),
        }
    };
}

fn order(msb: bool, n: usize, i: usize) -> usize {
    if msb { n - 1 - i } else { i }
}
fn enc(bit: usize) -> usize {
    match bit {
        1 | 2 | 4 => 1,
        3 | 6 => 3,
        5 => 5,
        _ => unreachable!(),
    }
}
fn dec(bit: usize) -> usize { enc(bit) * 8 / bit }

fn encode_block<B: Base>(base: B, symbols: &[u8; 256], input: &[u8],
                         output: &mut [u8]) {
    let bit = base.bit();
    let msb = base.msb();
    let mut x = 0u64;
    for i in 0 .. input.len() {
        x |= (input[i] as u64) << 8 * order(msb, enc(bit), i);
    }
    for i in 0 .. output.len() {
        let y = x >> bit * order(msb, dec(bit), i);
        output[i] = symbols[y as usize % 256];
    }
}
fn encode_mut<B: Base>(base: B, symbols: &[u8; 256], input: &[u8],
                       output: &mut [u8]) {
    let enc = enc(base.bit());
    let dec = dec(base.bit());
    let n = input.len() / enc;
    for i in 0 .. n {
        let input = unsafe { chunk_unchecked(input, enc, i) };
        let output = unsafe { chunk_mut_unchecked(output, dec, i) };
        encode_block(base, symbols, input, output);
    }
    encode_block(base, symbols, &input[enc * n ..], &mut output[dec * n ..]);
}

fn decode_block<B: Base>(base: B, values: &[u8; 256], input: &[u8],
                         output: &mut [u8]) -> Result<(), usize> {
    let bit = base.bit();
    let msb = base.msb();
    let mut x = 0u64;
    for j in 0 .. input.len() {
        let y = values[input[j] as usize];
        check!(j, y < 1 << bit);
        x |= (y as u64) << bit * order(msb, dec(bit), j);
    }
    for j in 0 .. output.len() {
        output[j] = (x >> 8 * order(msb, enc(bit), j)) as u8;
    }
    Ok(())
}
fn decode_mut<B: Base>(base: B, values: &[u8; 256], input: &[u8],
                       output: &mut [u8]) -> Result<(), usize> {
    let enc = enc(base.bit());
    let dec = dec(base.bit());
    let n = input.len() / dec;
    for i in 0 .. n {
        let input = unsafe { chunk_unchecked(input, dec, i) };
        let output = unsafe { chunk_mut_unchecked(output, enc, i) };
        decode_block(base, values, input, output).map_err(|e| dec * i + e)?;
    }
    decode_block(base, values, &input[dec * n ..], &mut output[enc * n ..])
        .map_err(|e| dec * n + e)
}
fn check_trail<B: Base>(base: B, ctb: bool, values: &[u8; 256], input: &[u8])
                        -> Result<(), ()> {
    if !ctb { return Ok(()) }
    let trail = base.bit() * input.len() % 8;
    if trail == 0 { return Ok(()) }
    let mut mask = (1 << trail) - 1;
    if !base.msb() { mask <<= base.bit() - trail; }
    check!((), values[input[input.len() - 1] as usize] & mask == 0);
    Ok(())
}
fn check_pad<B: Base>(base: B, pad: u8, input: &[u8]) -> Result<usize, usize> {
    let bit = base.bit();
    debug_assert_eq!(input.len(), dec(bit));
    let count = input.iter().rev().take_while(|&x| *x == pad).count();
    let len = input.len() - count;
    check!(len, len > 0 && bit * len % 8 < bit);
    Ok(len)
}

macro_rules! make_array {
    ($name: ident, $len: expr) => {
        impl std::ops::Deref for $name {
            type Target = [u8; $len];
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
        impl Clone for $name { fn clone(&self) -> Self { *self } }
        impl Copy for $name { }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                <&[u8] as std::fmt::Debug>::fmt(&(&self.0 as &[u8]), f)
            }
        }
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                &self.0 as &[u8] == &other.0 as &[u8]
            }
        }
        impl Eq for $name { }
    };
}

/// Convenience wrapper for `[u8; 128]`
///
/// Behaves as `[u8; 128]` through `Deref` and `DerefMut`, but also
/// implements `Clone` and other traits.
pub struct Array128([u8; 128]);
make_array!(Array128, 128);

struct Array256([u8; 256]);
make_array!(Array256, 256);

/// Order in which bits are read from a byte
///
/// # Examples
///
/// In the following example, we can see that a base with the
/// `MostSignificantFirst` bit-order has the most significant bit first in the
/// encoded output. In particular, the output is in the same order as the bits
/// in the byte. The opposite happens with the `LeastSignificantFirst`
/// bit-order. The least significant bit is first and the output is in the
/// reverse order.
///
/// ```rust
/// use data_encoding::Builder;
/// let mut builder = Builder::new(b"01");
/// let msb = builder.no_pad().unwrap();
/// let lsb = builder.least_significant_bit_first().no_pad().unwrap();
/// assert_eq!(msb.encode(&[0b01010011]), "01010011");
/// assert_eq!(lsb.encode(&[0b01010011]), "11001010");
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BitOrder {
    /// Most significant bit first
    ///
    /// This is the most common and most intuitive bit-order. In particular,
    /// this is the bit-order used by [RFC4648] and thus the usual hexadecimal,
    /// base64, base32, base64url, and base32hex encodings. This is the default
    /// bit-order when [building](struct.Builder.html) a base.
    ///
    /// [RFC4648]: https://tools.ietf.org/html/rfc4648
    MostSignificantFirst,

    /// Least significant bit first
    ///
    /// # Examples
    ///
    /// Here is how one would implement the [DNSCurve] base32 encoding:
    ///
    /// ```rust
    /// use data_encoding::Builder;
    /// let dns_curve = Builder::new(b"0123456789bcdfghjklmnpqrstuvwxyz")
    ///     .translate(b"BCDFGHJKLMNPQRSTUVWXYZ", b"bcdfghjklmnpqrstuvwxyz")
    ///     .least_significant_bit_first().no_pad().unwrap();
    /// assert_eq!(dns_curve.encode(&[0x64, 0x88]), "4321");
    /// assert_eq!(dns_curve.decode(b"4321").unwrap(), vec![0x64, 0x88]);
    /// ```
    ///
    /// [DNSCurve]: https://dnscurve.org/in-implement.html
    LeastSignificantFirst,
}
use BitOrder::*;

/// Base-conversion encoding (without padding)
///
/// # Theory
///
/// The main idea of a [base-conversion] encoding is to see `[u8]` as numbers
/// written in little-endian base256 and convert them in another little-endian
/// base. For performance reasons, this crate restricts this other base to be of
/// size 2 (binary), 4 (base4), 8 (octal), 16 (hexadecimal), 32 (base32), or 64
/// (base64). The converted number is written as `[u8]` although it doesn't use
/// all the 256 possible values of `u8`. This crate encodes to ASCII, so only
/// values smaller than 128 are allowed.
///
/// More precisely, we need the following elements:
///
/// - The bit-width N: 1 for binary, 2 for base4, 3 for octal, 4 for
/// hexadecimal, 5 for base32, and 6 for base64
/// - The [bit-order](enum.BitOrder.html): most or least significant bit first
/// - The symbols function S from [0, 2<sup>N</sup>) (called values and written
/// `uN`) to symbols (represented as `u8` although only ASCII symbols are
/// allowed, i.e. smaller than 128)
/// - The values partial function V from ASCII to [0, 2<sup>N</sup>), i.e. from
/// `u8` to `uN`
/// - Whether trailing bits are checked: trailing bits are leading zeros in
/// theory, but since numbers are little-endian they come last
///
/// For the encoding to be correct (i.e. encoding then decoding gives back the
/// initial input), V(S(i)) must be defined and equal to i for all i in [0,
/// 2<sup>N</sup>). For the encoding to be [canonical][canonical] (i.e.
/// different inputs decode to different outputs), trailing bits must be checked
/// and if V(i) is defined then S(V(i)) is equal to i for all i.
///
/// Encoding and decoding are given by the following pipeline:
///
/// ```text
/// [u8] <--1--> [[bit; 8]] <--2--> [[bit; N]] <--3--> [uN] <--4--> [u8]
/// 1: Map bit-order between each u8 and [bit; 8]
/// 2: Base conversion between base 2^8 and base 2^N (check trailing bits)
/// 3: Map bit-order between each [bit; N] and [uN]
/// 4: Map symbols/values between each uN and u8 (values must be defined)
/// ```
///
/// # Practice
///
/// ```rust
/// use data_encoding::Builder;
/// let binary = Builder::new(b"01").no_pad().unwrap();
/// let octal = Builder::new(b"01234567").no_pad().unwrap();
/// let hexadecimal = Builder::new(b"0123456789abcdef").no_pad().unwrap();
/// assert_eq!(binary.encode(b"Bit"), "010000100110100101110100");
/// assert_eq!(octal.encode(b"Bit"), "20464564");
/// assert_eq!(hexadecimal.encode(b"Bit"), "426974");
/// ```
///
/// The `binary` base has 2 symbols `0` and `1` with value 0 and 1 respectively.
/// The `octal` base has 8 symbols `0` to `7` with value 0 to 7. The
/// `hexadecimal` base has 16 symbols `0` to `9` and `a` to `f` with value 0 to
/// 15. The following diagram gives the idea of how encoding works in the
/// previous example (note that we can actually write such diagram only because
/// the bit-order is most significant first):
///
/// ```text
/// [      octal] |  2  :  0  :  4  :  6  :  4  :  5  :  6  :  4  |
/// [     binary] |0 1 0 0 0 0 1 0|0 1 1 0 1 0 0 1|0 1 1 1 0 1 0 0|
/// [hexadecimal] |   4   :   2   |   6   :   9   |   7   :   4   |
///                ^-- LSB                                       ^-- MSB
/// ```
///
/// Note that in theory, these little-endian numbers are read from right to left
/// (the most significant bit is at the right). Since leading zeros are
/// meaningless (in our usual decimal notation 0123 is the same as 123), it
/// explains why trailing bits must be zero. Trailing bits may occur when the
/// bit-width of a base does not divide 8. Only binary, base4, and hexadecimal
/// don't have trailing bits issues. So let's consider octal and base64, which
/// have trailing bits in similar circumstances:
///
/// ```rust
/// use data_encoding::{BASE64, Builder};
/// let octal = Builder::new(b"01234567").no_pad().unwrap();
/// assert_eq!(BASE64.no_pad().encode(b"B"), "Qg");
/// assert_eq!(octal.encode(b"B"), "204");
/// ```
///
/// We have the following diagram, where the base64 values are written between
/// parentheses:
///
/// ```text
/// [base64] |   Q(16)   :   g(32)   : [has 4 zero trailing bits]
/// [ octal] |  2  :  0  :  4  :       [has 1 zero trailing bit ]
///          |0 1 0 0 0 0 1 0|0 0 0 0
/// [ ascii] |       B       |
///                           ^-^-^-^-- leading zeros / trailing bits
/// ```
///
/// [base-conversion]: https://en.wikipedia.org/wiki/Positional_notation#Base_conversion
/// [canonical]: https://tools.ietf.org/html/rfc4648#section-3.5
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NoPad {
    sym: Array256,
    val: Array256,
    bit: u8,
    msb: bool,
    ctb: bool,
}

impl NoPad {
    fn bit(&self) -> usize { self.bit as usize }

    /// Returns the encoded length of an input of length `len`
    ///
    /// See [`encode_mut`] for when to use it.
    ///
    /// [`encode_mut`]: struct.NoPad.html#method.encode_mut
    pub fn encode_len(&self, len: usize) -> usize {
        div_ceil(8 * len, self.bit())
    }

    /// Encodes `input` in `output`
    ///
    /// # Panics
    ///
    /// Panics if `output`'s length does not match the result of [`encode_len`]
    /// for `input`'s length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use data_encoding::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// # let base64 = BASE64.no_pad();
    /// # let input = b"Hello world";
    /// let output = &mut buffer[0 .. base64.encode_len(input.len())];
    /// base64.encode_mut(input, output);
    /// # assert_eq!(output, b"SGVsbG8gd29ybGQ");
    /// ```
    ///
    /// [`encode_len`]: struct.NoPad.html#method.encode_len
    pub fn encode_mut(&self, input: &[u8], output: &mut [u8]) {
        assert_eq!(output.len(), self.encode_len(input.len()));
        dispatch!(encode_mut; self.bit(), self.msb, &self.sym, input, output)
    }

    /// Returns encoded `input`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// assert_eq!(BASE64.no_pad().encode(b"Hello world"), "SGVsbG8gd29ybGQ");
    /// ```
    pub fn encode(&self, input: &[u8]) -> String {
        let mut output = vec![0u8; self.encode_len(input.len())];
        self.encode_mut(input, &mut output);
        unsafe { String::from_utf8_unchecked(output) }
    }

    /// Returns the decoded length of an input of length `len`
    ///
    /// See [`decode_mut`] for when to use it.
    ///
    /// # Errors
    ///
    /// Returns an error if `len` is invalid. The error kind is [`Length`] and
    /// the error [position] is the greatest valid length smaller than `len`.
    ///
    /// [`decode_mut`]: struct.NoPad.html#method.decode_mut
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [position]: struct.DecodeError.html#structfield.position
    pub fn decode_len(&self, len: usize) -> Result<usize, DecodeError> {
        let bit = self.bit();
        let trail = bit * len % 8;
        check!(DecodeError { position: len - trail / bit,
                             kind: DecodeKind::Length }, trail < bit);
        Ok(bit * len / 8)
    }

    /// Decodes `input` in `output`
    ///
    /// # Panics
    ///
    /// Panics if `output`'s length does not match the result of [`decode_len`]
    /// for `input`'s length. Also panics if `decode_len` fails for `input`'s
    /// length.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be [`Symbol`]
    /// or [`Trailing`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use data_encoding::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// # let base64 = BASE64.no_pad();
    /// # let input = b"SGVsbG8gd29ybGQ";
    /// let output = &mut buffer[0 .. base64.decode_len(input.len()).unwrap()];
    /// base64.decode_mut(input, output).unwrap();
    /// # assert_eq!(output, b"Hello world");
    /// ```
    ///
    /// [`decode_len`]: struct.NoPad.html#method.decode_len
    /// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
    /// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
    pub fn decode_mut(&self, input: &[u8], output: &mut [u8])
                      -> Result<(), DecodeError> {
        assert_eq!(output.len(), self.decode_len(input.len()).unwrap());
        dispatch!(decode_mut; self.bit(), self.msb, &self.val, input, output)
            .map_err(|pos| DecodeError { position: pos,
                                         kind: DecodeKind::Symbol })?;
        dispatch!(check_trail; self.bit(), self.msb, self.ctb, &self.val, input)
            .map_err(|()| DecodeError { position: input.len() - 1,
                                        kind: DecodeKind::Trailing })?;
        Ok(())
    }

    /// Returns decoded `input`
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be
    /// [`Length`], [`Symbol`], or [`Trailing`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// assert_eq!(BASE64.no_pad().decode(b"SGVsbG8gd29ybGQ").unwrap(),
    ///            b"Hello world");
    /// ```
    ///
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
    /// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
    pub fn decode(&self, input: &[u8]) -> Result<Vec<u8>, DecodeError> {
        let mut output = vec![0u8; self.decode_len(input.len())?];
        self.decode_mut(input, &mut output)?;
        Ok(output)
    }

    /// Returns the bit-width
    pub fn bit_width(&self) -> usize { self.bit() }

    /// Returns the bit-order
    pub fn bit_order(&self) -> BitOrder {
        if self.msb { MostSignificantFirst } else { LeastSignificantFirst }
    }

    /// Returns the symbols
    pub fn symbols(&self) -> &str {
        let symbols = &self.sym[0 .. 1 << self.bit()];
        unsafe { std::str::from_utf8_unchecked(symbols) }
    }

    /// Returns the non-canonical symbols
    ///
    /// Non-canonical symbols are ASCII characters i for which V(i) is defined
    /// but S(V(i)) is different from i. In other words, these characters cannot
    /// be produced by the encoding function but are still recognized by the
    /// decoding function and behave as the canonical symbol of the same value.
    ///
    /// The result `(from, to)` has the following properties:
    ///
    /// - `from` and `to` are ASCII and have the same length
    /// - All non-canonical symbols are listed in `from` in ascending order
    /// - `from[i]` is a non-canonical symbol that behaves as `to[i]` for all i
    ///
    /// # Examples
    ///
    /// ```rust
    /// let (from, to) = data_encoding::HEXLOWER_PERMISSIVE.translate();
    /// assert_eq!((from.as_str(), to.as_str()), ("ABCDEF", "abcdef"));
    /// ```
    pub fn translate(&self) -> (String, String) {
        let mut from = vec![];
        let mut to = vec![];
        for i in 0 .. 128 {
            if self.val[i] == 128 { continue; }
            let canonical = self.sym[self.val[i] as usize];
            if i as u8 == canonical { continue; }
            from.push(i as u8);
            to.push(canonical);
        }
        let from = unsafe { String::from_utf8_unchecked(from) };
        let to = unsafe { String::from_utf8_unchecked(to) };
        (from, to)
    }

    /// Whether trailing bits are checked
    ///
    /// Returns `None` for bases that don't need to check trailing bits (like
    /// base2, base4, and base16). Otherwise, for bases that would need it (like
    /// base8, base32, and base64), returns whether trailing bits are checked.
    pub fn check_trailing_bits(&self) -> Option<bool> {
        if 8 % self.bit() == 0 { None } else { Some(self.ctb) }
    }
}

/// Padded base-conversion encoding
///
/// The padded encoding extends the [base-conversion] encoding. This is only
/// useful for octal, base32, and base64. And for those bases, it is only useful
/// if the length of the data to encode is not known in advance.
///
/// # Theory
///
/// Bases for which the bit-width N does not divide 8 may not concatenate
/// encoded data. This comes from the fact that it is not possible to make the
/// difference between trailing bits and encoding bits. Padding solves this
/// issue by adding a new character (which is not a symbol) to discriminate
/// between trailing bits and encoding bits. The idea is to work by blocks of
/// lcm(8, N) bits, where lcm(8, N) is the least common multiple of 8 and N.
/// When such block is not complete, it is padded.
///
/// # Practice
///
/// For octal and base64, lcm(8, 3) == lcm(8, 6) == 24 bits or 3 bytes. For
/// base32, lcm(8, 5) is 40 bits or 5 bytes. Let's consider octal and base64:
///
/// ```rust
/// use data_encoding::{BASE64, Builder};
/// let octal = Builder::new(b"01234567").pad(b'=').padded().unwrap();
/// // We start encoding but we only have "B" for now.
/// assert_eq!(BASE64.encode(b"B"), "Qg==");
/// assert_eq!(octal.encode(b"B"), "204=====");
/// // Now we have "it".
/// assert_eq!(BASE64.encode(b"it"), "aXQ=");
/// assert_eq!(octal.encode(b"it"), "322720==");
/// // By concatenating everything, we may decode the original data.
/// assert_eq!(BASE64.decode_concat(b"Qg==aXQ=").unwrap(), b"Bit");
/// assert_eq!(octal.decode_concat(b"204=====322720==").unwrap(), b"Bit");
/// ```
///
/// We have the following diagrams:
///
/// ```text
/// [base64] |   Q(16)   :   g(32)   :     =     :     =     |
/// [ octal] |  2  :  0  :  4  :  =  :  =  :  =  :  =  :  =  |
///          |0 1 0 0 0 0 1 0|. . . . . . . .|. . . . . . . .|
/// [ ascii] |       B       |        end of block aligned --^
///          ^-- beginning of block aligned
///
/// [base64] |   a(26)   :   X(23)   :   Q(16)   :     =     |
/// [ octal] |  3  :  2  :  2  :  7  :  2  :  0  :  =  :  =  |
///          |0 1 1 0 1 0 0 1|0 1 1 1 0 1 0 0|. . . . . . . .|
/// [ ascii] |       i       |       t       |
/// ```
///
/// [base-conversion]: struct.NoPad.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Padded {
    no_pad: NoPad,
    pad: u8,
}

impl Padded {
    /// Returns the encoded length of an input of length `len`
    ///
    /// See [`encode_mut`] for when to use it.
    ///
    /// [`encode_mut`]: struct.Padded.html#method.encode_mut
    pub fn encode_len(&self, len: usize) -> usize {
        let bit = self.no_pad.bit();
        div_ceil(len, enc(bit)) * dec(bit)
    }

    /// Encodes `input` in `output`.
    ///
    /// # Panics
    ///
    /// Panics if `output`'s length does not match the result of [`encode_len`]
    /// for `input`'s length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// let input = b"Hello world";
    /// let output = &mut buffer[0 .. BASE64.encode_len(input.len())];
    /// BASE64.encode_mut(input, output);
    /// assert_eq!(output, b"SGVsbG8gd29ybGQ=");
    /// ```
    ///
    /// [`encode_len`]: struct.Padded.html#method.encode_len
    pub fn encode_mut(&self, input: &[u8], output: &mut [u8]) {
        assert_eq!(output.len(), self.encode_len(input.len()));
        let last = self.no_pad.encode_len(input.len());
        self.no_pad.encode_mut(input, &mut output[0 .. last]);
        for i in output[last ..].iter_mut() {
            *i = self.pad;
        }
    }

    /// Returns encoded `input`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// assert_eq!(BASE64.encode(b"Hello world"), "SGVsbG8gd29ybGQ=");
    /// ```
    pub fn encode(&self, input: &[u8]) -> String {
        let mut output = vec![0u8; self.encode_len(input.len())];
        self.encode_mut(input, &mut output);
        unsafe { String::from_utf8_unchecked(output) }
    }

    /// Returns the decoded length of an input of length `len`
    ///
    /// See [`decode_mut`] for when to use it.
    ///
    /// # Errors
    ///
    /// Returns an error if `len` is invalid. The error kind is [`Length`] and
    /// the error [position] is the greatest valid length smaller than `len`.
    ///
    /// [`decode_mut`]: struct.Padded.html#method.decode_mut
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [position]: struct.DecodeError.html#structfield.position
    pub fn decode_len(&self, len: usize) -> Result<usize, DecodeError> {
        let bit = self.no_pad.bit();
        check!(DecodeError { position: floor(len, dec(bit)),
                             kind: DecodeKind::Length },
               len % dec(bit) == 0);
        Ok(len / dec(bit) * enc(bit))
    }

    /// Decodes `input` in `output`
    ///
    /// Returns the length of the decoded output. This length may be smaller
    /// than output's length if the input is padded. The output bytes after the
    /// returned length are not initialized and should not be read.
    ///
    /// # Panics
    ///
    /// Panics if `output`'s length does not match the result of [`decode_len`]
    /// for `input`'s length. Also panics if `decode_len` fails for `input`'s
    /// length.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be
    /// [`Symbol`], [`Trailing`], or [`Padding`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// let input = b"SGVsbG8gd29ybGQ=";
    /// let output = &mut buffer[0 .. BASE64.decode_len(input.len()).unwrap()];
    /// let len = BASE64.decode_mut(input, output).unwrap();
    /// assert_eq!(&output[0 .. len], b"Hello world");
    /// ```
    ///
    /// [`decode_len`]: struct.Padded.html#method.decode_len
    /// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
    /// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
    /// [`Padding`]: enum.DecodeKind.html#variant.Padding
    pub fn decode_mut(&self, input: &[u8], output: &mut [u8])
                      -> Result<usize, DecodeError> {
        if input.len() == 0 { return Ok(0); }
        assert_eq!(output.len(), self.decode_len(input.len()).unwrap());
        let dec = dec(self.no_pad.bit());
        let ilen = input.len() - dec;
        let irem = dispatch!(check_pad; self.no_pad.bit(), self.no_pad.msb,
                             self.pad, &input[ilen ..])
            .map_err(|e| DecodeError { position: ilen + e,
                                       kind: DecodeKind::Padding })?;
        let olen = self.no_pad.decode_len(ilen + irem).unwrap();
        self.no_pad.decode_mut(&input[.. ilen + irem], &mut output[.. olen])?;
        Ok(olen)
    }

    /// Returns decoded `input`
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be
    /// [`Length`], [`Symbol`], [`Trailing`], or [`Padding`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// assert_eq!(BASE64.decode(b"SGVsbG8gd29ybGQ=").unwrap(), b"Hello world");
    /// ```
    ///
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
    /// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
    /// [`Padding`]: enum.DecodeKind.html#variant.Padding
    pub fn decode(&self, input: &[u8]) -> Result<Vec<u8>, DecodeError> {
        let mut output = vec![0u8; self.decode_len(input.len())?];
        let len = self.decode_mut(input, &mut output)?;
        output.truncate(len);
        Ok(output)
    }

    /// Decodes concatenated `input` in `output`
    ///
    /// Returns the length of the decoded output. This length may be smaller
    /// than output's length if the input contained padding. The output bytes
    /// after the returned length are not initialized and should not be read.
    ///
    /// # Panics
    ///
    /// Panics if `output`'s length does not match the result of [`decode_len`]
    /// for `input`'s length. Also panics if `decode_len` fails for `input`'s
    /// length.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be
    /// [`Symbol`], [`Trailing`], or [`Padding`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// let input = b"SGVsbA==byB3b3JsZA==";
    /// let output = &mut buffer[0 .. BASE64.decode_len(input.len()).unwrap()];
    /// let len = BASE64.decode_concat_mut(input, output).unwrap();
    /// assert_eq!(&output[0 .. len], b"Hello world");
    /// ```
    ///
    /// [`decode_len`]: struct.Padded.html#method.decode_len
    /// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
    /// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
    /// [`Padding`]: enum.DecodeKind.html#variant.Padding
    pub fn decode_concat_mut(&self, input: &[u8], output: &mut [u8])
                             -> Result<usize, DecodeError> {
        assert_eq!(output.len(), self.decode_len(input.len()).unwrap());
        let bit = self.no_pad.bit();
        let enc = enc(bit);
        let dec = dec(bit);
        let mut inpos = 0;
        let mut outpos = 0;
        let mut outend = output.len();
        while inpos < input.len() {
            let ret = self.no_pad.decode_mut(
                &input[inpos ..], &mut output[outpos .. outend]);
            match ret {
                Ok(()) => break,
                Err(err) => {
                    debug_assert_eq!(err.kind, DecodeKind::Symbol);
                    inpos += err.position / dec * dec;
                    outpos += err.position / dec * enc;
                },
            }
            let inlen = dispatch!(check_pad; self.no_pad.bit(), self.no_pad.msb,
                                  self.pad, &input[inpos .. inpos + dec])
                .map_err(|e| DecodeError { position: inpos + e,
                                           kind: DecodeKind::Padding })?;
            let outlen = self.no_pad.decode_len(inlen).unwrap();
            self.no_pad.decode_mut(&input[inpos .. inpos + inlen],
                                   &mut output[outpos .. outpos + outlen])
                .map_err(|mut e| { e.position += inpos; e })?;
            inpos += dec;
            outpos += outlen;
            outend -= enc - outlen;
        }
        Ok(outend)
    }

    /// Returns decoded concatenated `input`
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be
    /// [`Length`], [`Symbol`], [`Trailing`], or [`Padding`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// assert_eq!(BASE64.decode_concat(b"SGVsbA==byB3b3JsZA==").unwrap(),
    ///            b"Hello world");
    /// ```
    ///
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
    /// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
    /// [`Padding`]: enum.DecodeKind.html#variant.Padding
    pub fn decode_concat(&self, input: &[u8]) -> Result<Vec<u8>, DecodeError> {
        let mut output = vec![0u8; self.decode_len(input.len())?];
        let len = self.decode_concat_mut(input, &mut output)?;
        output.truncate(len);
        Ok(output)
    }

    /// Returns the associated base-conversion encoding
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding::BASE64;
    /// assert_eq!(BASE64.encode(b"Helo"), "SGVsbw==");
    /// assert_eq!(BASE64.no_pad().encode(b"Helo"), "SGVsbw");
    /// ```
    pub fn no_pad(&self) -> &NoPad { &self.no_pad }

    /// Returns the padding character
    pub fn padding(&self) -> u8 { self.pad }
}

/// Base representation
///
/// Convenience methods are provided to edit the fields, although they may be
/// manually edited.
///
/// # Examples
///
/// See the [lower-case hexadecimal][1], [upper-case hexadecimal][2],
/// [lower-case permissive hexadecimal][3], [base32], [base32hex], [base64], and
/// [base64url] encodings.
///
/// [1]: constant.HEXLOWER.html
/// [2]: constant.HEXUPPER.html
/// [3]: constant.HEXLOWER_PERMISSIVE.html
/// [base32]: constant.BASE32.html
/// [base32hex]: constant.BASE32HEX.html
/// [base64]: constant.BASE64.html
/// [base64url]: constant.BASE64URL.html
#[derive(Debug, Clone)]
pub struct Builder {
    /// Symbols
    ///
    /// The number of symbols must be 2, 4, 8, 16, 32, or 64. Symbols must be
    /// ASCII characters (smaller than 128) and they must be unique.
    pub symbols: Box<[u8]>,

    /// Values
    ///
    /// A value of 128 means that the index is not a symbol. In other words, if
    /// `values[s] != 128` then `s` is a symbol (canonical if
    /// `symbols[values[s]]` is equal to `s`) and `values[s]` is its value.
    ///
    /// Default is the inverse of symbols.
    pub values: Box<Array128>,

    /// Bit-order
    ///
    /// Default is most significant bit first.
    pub bit_order: BitOrder,

    /// Padding
    ///
    /// Default is no padding.
    pub padding: Option<u8>,

    /// Whether trailing bits are checked
    ///
    /// Default is to check trailing bits. This field is ignored when
    /// unnecessary (i.e. for base2, base4, and base16).
    pub check_trailing_bits: bool,
}

#[derive(Debug, Copy, Clone)]
enum BuilderErrorImpl {
    BadSize,
    BadSym(u8),
    BadVal(u8),
    BadPad(Option<u8>),
}
use BuilderErrorImpl::*;

/// Base building error
#[derive(Debug, Copy, Clone)]
pub struct BuilderError(BuilderErrorImpl);

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            BadSize => write!(f, "invalid number of symbols"),
            BadSym(s) => write!(f, "non-ascii symbol {:#x}", s),
            BadVal(s) => write!(f, "invalid value for {:?}", s as char),
            BadPad(Some(s)) if s < 128 => write!(f, "padding symbol conflict"),
            BadPad(Some(pad)) => write!(f, "non-ascii padding {:#x}", pad),
            BadPad(None) => write!(f, "unnecessary or missing padding"),
        }
    }
}

impl std::error::Error for BuilderError {
    fn description(&self) -> &str {
        match self.0 {
            BadSize => "invalid number of symbols",
            BadSym(_) => "non-ascii symbol",
            BadVal(_) => "invalid value",
            BadPad(Some(s)) if s < 128 => "padding symbol conflict",
            BadPad(Some(_)) => "non-ascii padding",
            BadPad(None) => "unnecessary or missing padding",
        }
    }
}

impl Builder {
    fn bit(&self) -> Result<u8, BuilderError> {
        match self.symbols.len() {
            2 => Ok(1),
            4 => Ok(2),
            8 => Ok(3),
            16 => Ok(4),
            32 => Ok(5),
            64 => Ok(6),
            _ => Err(BuilderError(BadSize)),
        }
    }

    fn check(&self) -> Result<(), BuilderError> {
        let bit = self.bit()?;
        let even = 8 % bit == 0;
        check!(BuilderError(BadPad(None)), self.padding.is_none() || !even);
        for v in 0 .. self.symbols.len() {
            let s = self.symbols[v];
            check!(BuilderError(BadSym(s)), s < 128);
            check!(BuilderError(BadVal(s)), self.values[s as usize] == v as u8);
        }
        for s in 0 .. self.values.len() {
            if self.values[s] == 128 { continue; }
            check!(BuilderError(BadVal(s as u8)), self.values[s] < 1 << bit);
        }
        if let Some(pad) = self.padding {
            check!(BuilderError(BadPad(Some(pad))), pad < 128);
            for s in 0 .. self.values.len() {
                if self.values[s] == 128 { continue; }
                check!(BuilderError(BadPad(Some(pad))), pad != s as u8);
            }
        }
        Ok(())
    }

    fn no_pad_unchecked(&self) -> NoPad {
        let bit = self.bit().unwrap();
        let mut base = NoPad {
            sym: Array256([0; 256]), val: Array256([128; 256]), bit: bit,
            msb: self.bit_order == MostSignificantFirst,
            ctb: 8 % bit != 0 && self.check_trailing_bits,
        };
        for i in 0 .. base.sym.len() {
            base.sym[i] = self.symbols[i % self.symbols.len()];
        }
        for i in 0 .. self.values.len() {
            base.val[i] = self.values[i];
        }
        base
    }

    /// Returns a canonical base representation for `symbols`
    ///
    /// By default, the base representation does not have non-canonical symbols.
    /// It does not have padding. It is most significant bit first. And it
    /// checks the trailing bits if necessary.
    ///
    /// # Errors
    ///
    /// Errors are silently ignored. In other words, if a symbol is not ASCII,
    /// if there are duplicate symbols, or if the number of symbols is not a
    /// power of 2 smaller than 128, then no errors are signaled. However, when
    /// building the base with [`no_pad`] or [`padded`], if the base
    /// representation is still invalid, an error will be returned.
    ///
    /// [`no_pad`]: struct.Builder.html#method.no_pad
    /// [`padded`]: struct.Builder.html#method.padded
    pub fn new(symbols: &[u8]) -> Builder {
        let mut builder = Builder {
            symbols: symbols.to_vec().into_boxed_slice(),
            values: Box::new(Array128([128; 128])),
            bit_order: MostSignificantFirst,
            padding: None,
            check_trailing_bits: true,
        };
        for v in 0 .. symbols.len() {
            if symbols[v] >= 128 { continue; }
            builder.values[symbols[v] as usize] = v as u8;
        }
        builder
    }

    /// Sets padding
    pub fn pad(&mut self, pad: u8) -> &mut Builder {
        self.padding = Some(pad);
        self
    }

    /// Adds non-canonical symbols
    ///
    /// For all i, `from[i]` is given the same value as `to[i]`.
    ///
    /// By default there are only canonical symbols. Non-canonical symbols
    /// cannot be produced by encoding functions, but they are recognized by
    /// decoding functions. They behave as the canonical symbol of the same
    /// value.
    ///
    /// # Panics
    ///
    /// Panics if `from` and `to` don't have the same length.
    ///
    /// # Errors
    ///
    /// Errors are silently ignored. If a character in `from` or `to` is not
    /// ASCII then the pair is skipped. If the character in `from` is already a
    /// symbol it is overwritten. If the character in `to` is not a symbol,
    /// `from` is reset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use data_encoding::Builder;
    /// let base = Builder::new(b"0123456789abcdef")
    ///     .translate(b"ABCDEF", b"abcdef").no_pad().unwrap();
    /// assert_eq!(base.decode(b"Bb").unwrap(), vec![0xbb]);
    /// ```
    pub fn translate(&mut self, from: &[u8], to: &[u8]) -> &mut Builder {
        assert_eq!(from.len(), to.len());
        for i in 0 .. from.len() {
            if from[i] >= 128 || to[i] >= 128 { continue; }
            self.values[from[i] as usize] = self.values[to[i] as usize];
        }
        self
    }

    /// Sets bit-order to least significant bit first
    pub fn least_significant_bit_first(&mut self) -> &mut Builder {
        self.bit_order = LeastSignificantFirst;
        self
    }

    /// Ignores trailing bits
    pub fn ignore_trailing_bits(&mut self) -> &mut Builder {
        self.check_trailing_bits = false;
        self
    }

    /// Returns the represented base-conversion encoding
    ///
    /// # Errors
    ///
    /// Returns an error if the base representation is invalid.
    pub fn no_pad(&self) -> Result<NoPad, BuilderError> {
        check!(BuilderError(BadPad(None)), self.padding.is_none());
        self.check()?;
        Ok(self.no_pad_unchecked())
    }

    /// Returns the represented padded base-conversion encoding
    ///
    /// # Errors
    ///
    /// Returns an error if the base representation is invalid.
    pub fn padded(&self) -> Result<Padded, BuilderError> {
        let pad = self.padding.ok_or(BuilderError(BadPad(None)))?;
        self.check()?;
        Ok(Padded { no_pad: self.no_pad_unchecked(), pad: pad })
    }
}

impl<'a> From<&'a NoPad> for Builder {
    fn from(no_pad: &NoPad) -> Builder {
        let mut builder = Builder {
            symbols: no_pad.symbols().as_bytes().to_vec().into_boxed_slice(),
            values: Box::new(Array128([0; 128])),
            bit_order: no_pad.bit_order(),
            padding: None,
            check_trailing_bits: no_pad.ctb,
        };
        builder.values.copy_from_slice(&no_pad.val[0 .. 128]);
        builder
    }
}

impl<'a> From<&'a Padded> for Builder {
    fn from(padded: &Padded) -> Builder {
        let mut builder = Builder::from(&padded.no_pad);
        builder.padding = Some(padded.pad);
        builder
    }
}

const X_: u8 = 128;
macro_rules! make_val {
    ($($v: expr),*) => { [
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
macro_rules! make_sym {
    (7; $($s: expr),*) => { [ $($s),*, $($s),*, ] };
    (6; $($s: expr),*) => { make_sym!(7; $($s),*, $($s),*) };
    (5; $($s: expr),*) => { make_sym!(6; $($s),*, $($s),*) };
    (4; $($s: expr),*) => { make_sym!(5; $($s),*, $($s),*) };
}
macro_rules! make_base {
    ($b: tt; $($v: expr),*; $($s: expr),*;) => {
        NoPad {
            sym: Array256(make_sym!($b; $($s),*)),
            val: Array256(make_val!($($v),*)),
            bit: $b,
            msb: true,
            ctb: 8 % $b != 0,
        }
    };
    ($p: expr; $b: tt; $($v: expr),*; $($s: expr),*;) => {
        Padded {
            no_pad: make_base!($b; $($v),*; $($s),*;),
            pad: $p,
        }
    };
}

/// Lower-case hexadecimal encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{Builder, HEXLOWER};
/// assert_eq!(HEXLOWER, &Builder::new(b"0123456789abcdef").no_pad().unwrap());
/// ```
///
/// # Examples
///
/// ```rust
/// use data_encoding::HEXLOWER;
/// let deadbeef = vec![0xde, 0xad, 0xbe, 0xef];
/// assert_eq!(HEXLOWER.decode(b"deadbeef").unwrap(), deadbeef);
/// assert_eq!(HEXLOWER.encode(&deadbeef), "deadbeef");
/// ```
pub const HEXLOWER: &'static NoPad = HEXLOWER_IMPL;
const HEXLOWER_IMPL: &'static NoPad = &make_base!{
    4;
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0 , 1 , 2 , 3 , 4 , 5 , 6 , 7 , 8 , 9 , X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, 10, 11, 12, 13, 14, 15, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_;
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f';
};

/// RFC4648 hex encoding (upper-case hexadecimal encoding)
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{Builder, HEXUPPER};
/// assert_eq!(HEXUPPER, &Builder::new(b"0123456789ABCDEF").no_pad().unwrap());
/// ```
///
/// It is compliant with [RFC4648] and known as "base16" or "hex".
///
/// # Examples
///
/// ```rust
/// use data_encoding::HEXUPPER;
/// let deadbeef = vec![0xde, 0xad, 0xbe, 0xef];
/// assert_eq!(HEXUPPER.decode(b"DEADBEEF").unwrap(), deadbeef);
/// assert_eq!(HEXUPPER.encode(&deadbeef), "DEADBEEF");
/// ```
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-8
pub const HEXUPPER: &'static NoPad = HEXUPPER_IMPL;
const HEXUPPER_IMPL: &'static NoPad = &make_base!{
    4;
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0 , 1 , 2 , 3 , 4 , 5 , 6 , 7 , 8 , 9 , X_, X_, X_, X_, X_, X_,
    X_, 10, 11, 12, 13, 14, 15, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_;
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F';
};

/// Lower-case permissive hexadecimal encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{Builder, HEXLOWER_PERMISSIVE};
/// let mut base = Builder::new(b"0123456789abcdef")
///     .translate(b"ABCDEF", b"abcdef").no_pad().unwrap();
/// assert_eq!(HEXLOWER_PERMISSIVE, &base);
/// ```
///
/// # Examples
///
/// ```rust
/// use data_encoding::HEXLOWER_PERMISSIVE;
/// let deadbeef = vec![0xde, 0xad, 0xbe, 0xef];
/// assert_eq!(HEXLOWER_PERMISSIVE.decode(b"DeadBeef").unwrap(), deadbeef);
/// assert_eq!(HEXLOWER_PERMISSIVE.encode(&deadbeef), "deadbeef");
/// ```
///
/// You can also define a shorter name:
///
/// ```rust
/// use data_encoding::{HEXLOWER_PERMISSIVE, NoPad};
/// const HEX: &'static NoPad = HEXLOWER_PERMISSIVE;
/// ```
pub const HEXLOWER_PERMISSIVE: &'static NoPad = HEXLOWER_PERMISSIVE_IMPL;
const HEXLOWER_PERMISSIVE_IMPL: &'static NoPad = &make_base!{
    4;
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0 , 1 , 2 , 3 , 4 , 5 , 6 , 7 , 8 , 9 , X_, X_, X_, X_, X_, X_,
    X_, 10, 11, 12, 13, 14, 15, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, 10, 11, 12, 13, 14, 15, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_;
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f';
};

/// RFC4648 base32 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{Builder, BASE32};
/// assert_eq!(BASE32, &Builder::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567")
///                        .pad(b'=').padded().unwrap());
/// ```
///
/// It is conformant with [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-6
pub const BASE32: &'static Padded = BASE32_IMPL;
const BASE32_IMPL: &'static Padded = &make_base!{
    b'='; 5;
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, 26, 27, 28, 29, 30, 31, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, 0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, 10, 11, 12, 13, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_;
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H',
    b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X',
    b'Y', b'Z', b'2', b'3', b'4', b'5', b'6', b'7';
};

/// RFC4648 base32hex encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{Builder, BASE32HEX};
/// assert_eq!(BASE32HEX, &Builder::new(b"0123456789ABCDEFGHIJKLMNOPQRSTUV")
///                          .pad(b'=').padded().unwrap());
/// ```
///
/// It is conformant with [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-7
pub const BASE32HEX: &'static Padded = BASE32HEX_IMPL;
const BASE32HEX_IMPL: &'static Padded = &make_base!{
    b'='; 5;
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, X_, X_, X_, X_, X_, X_,
    X_, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 26, 27, 28, 29, 30, 31, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_,
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_;
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N',
    b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V';
};

/// RFC4648 base64 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{Builder, BASE64};
/// assert_eq!(BASE64, &Builder::new(
///     b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
///         .pad(b'=').padded().unwrap());
/// ```
///
/// It is conformant with [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-4
pub const BASE64: &'static Padded = BASE64_IMPL;
const BASE64_IMPL: &'static Padded = &make_base!{
    b'='; 6;
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, 62, X_, X_, X_, 63,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, X_, X_, X_, X_, X_, X_,
    X_, 0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, 10, 11, 12, 13, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, X_, X_, X_, X_, X_,
    X_, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, X_, X_, X_, X_, X_;
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H',
    b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X',
    b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
    b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3',
    b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/';
};

/// RFC4648 base64url encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{Builder, BASE64URL};
/// assert_eq!(BASE64URL, &Builder::new(
///     b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_")
///         .pad(b'=').padded().unwrap());
/// ```
///
/// It is conformant with [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-5
pub const BASE64URL: &'static Padded = BASE64URL_IMPL;
const BASE64URL_IMPL: &'static Padded = &make_base!{
    b'='; 6;
    X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, X_, 62, X_, X_,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, X_, X_, X_, X_, X_, X_,
    X_, 0_, 1_, 2_, 3_, 4_, 5_, 6_, 7_, 8_, 9_, 10, 11, 12, 13, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, X_, X_, X_, X_, 63,
    X_, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, X_, X_, X_, X_, X_;
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H',
    b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X',
    b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
    b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3',
    b'4', b'5', b'6', b'7', b'8', b'9', b'-', b'_';
};
