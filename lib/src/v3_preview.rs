//! Provides an unstable preview of the 3.0.0 version.
//!
//! This module is gated by the `v3-preview` feature. This feature and this module are unstable in
//! the sense that breaking changes are only considered minor changes instead of major changes. In
//! particular, you should use the tilde requirement `~2.6.0` instead of the default caret
//! requirement `2.6.0` (or explicitly `^2.6.0`). For more information, consult the [SemVer
//! compatibility][semver] section of the Cargo Book.
//!
//! This feature and this module also have a different MSRV: 1.70 instead of 1.48.
//!
//! [semver]: https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility

#![warn(let_underscore_drop)]
#![warn(unsafe_op_in_unsafe_fn)]
#![allow(clippy::incompatible_msrv)]
#![allow(clippy::match_bool)]

#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto as _};
use core::marker::PhantomData;
use core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
pub use crate::Specification;
use crate::{
    chunk_mut_unchecked, chunk_unchecked, dec, div_ceil, enc, floor, order, vectorize,
    InternalEncoding, IGNORE, PADDING,
};
pub use crate::{DecodeError, DecodeKind, DecodePartial, Encoding as DynEncoding};

/// Type-level bit-width of an encoding.
pub trait BitWidth: sealed::BitWidth {}

/// Type-level bool.
pub trait Bool: sealed::Bool {}

mod sealed {
    pub trait BitWidth {
        const VAL: usize;
    }

    pub trait Bool {
        type If<Then: Copy, Else: Copy>: Copy;
        const VAL: bool;
        fn open<Then: Copy, Else: Copy>(cond: Self::If<Then, Else>) -> If<Then, Else>;
        fn make<Then: Copy, Else: Copy>(
            then: impl FnOnce() -> Then, else_: impl FnOnce() -> Else,
        ) -> Self::If<Then, Else>;
    }

    #[derive(Debug, Copy, Clone)]
    pub enum If<Then, Else> {
        Then(Then),
        Else(Else),
    }
}
use sealed::If;

macro_rules! new_bit_width {
    ($(($N:ident, $v:expr, $b:literal),)*) => {
        $(
            #[doc = concat!(" Bit-width of ", $b, " encodings.")]
            #[derive(Debug)]
            pub enum $N {}
            impl BitWidth for $N {}
            impl sealed::BitWidth for $N { const VAL: usize = $v; }
        )*
    };
}
new_bit_width![
    (Bit1, 1, "base2"),
    (Bit2, 2, "base4"),
    (Bit3, 3, "base8"),
    (Bit4, 4, "base16"),
    (Bit5, 5, "base32"),
    (Bit6, 6, "base64"),
];

/// Type-level false.
#[derive(Debug)]
pub enum False {}
impl Bool for False {}
impl sealed::Bool for False {
    type If<Then: Copy, Else: Copy> = Else;
    const VAL: bool = false;
    fn open<Then: Copy, Else: Copy>(cond: Self::If<Then, Else>) -> If<Then, Else> {
        If::Else(cond)
    }
    fn make<Then: Copy, Else: Copy>(
        _then: impl FnOnce() -> Then, else_: impl FnOnce() -> Else,
    ) -> Self::If<Then, Else> {
        else_()
    }
}

/// Type-level true.
#[derive(Debug, Copy, Clone)]
pub enum True {}
impl Bool for True {}
impl sealed::Bool for True {
    type If<Then: Copy, Else: Copy> = Then;
    const VAL: bool = true;
    fn open<Then: Copy, Else: Copy>(cond: Self::If<Then, Else>) -> If<Then, Else> {
        If::Then(cond)
    }
    fn make<Then: Copy, Else: Copy>(
        then: impl FnOnce() -> Then, _else: impl FnOnce() -> Else,
    ) -> Self::If<Then, Else> {
        then()
    }
}

// TODO(https://github.com/rust-lang/rust/issues/79995): Use write_slice() instead.
unsafe fn copy_from_slice(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
    dst.copy_from_slice(unsafe { &*(src as *const [u8] as *const [MaybeUninit<u8>]) });
}

// TODO(https://github.com/rust-lang/rust/issues/63569): Use slice_assume_init_mut() instead.
unsafe fn slice_assume_init_mut(xs: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    unsafe { &mut *(xs as *mut [MaybeUninit<u8>] as *mut [u8]) }
}

unsafe fn slice_uninit_mut(xs: &mut [u8]) -> &mut [MaybeUninit<u8>] {
    unsafe { &mut *(xs as *mut [u8] as *mut [MaybeUninit<u8>]) }
}

#[cfg(feature = "alloc")]
fn reserve_spare(xs: &mut Vec<u8>, n: usize) -> &mut [MaybeUninit<u8>] {
    xs.reserve(n);
    &mut xs.spare_capacity_mut()[.. n]
}

fn encode_len<Bit: BitWidth>(len: usize) -> usize {
    div_ceil(8 * len, Bit::VAL)
}

fn encode_block<Bit: BitWidth, Msb: Bool>(
    symbols: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) {
    debug_assert!(input.len() <= enc(Bit::VAL));
    debug_assert_eq!(output.len(), encode_len::<Bit>(input.len()));
    let bit = Bit::VAL;
    let msb = Msb::VAL;
    let mut x = 0u64;
    for (i, input) in input.iter().enumerate() {
        x |= u64::from(*input) << (8 * order(msb, enc(bit), i));
    }
    for (i, output) in output.iter_mut().enumerate() {
        let y = x >> (bit * order(msb, dec(bit), i));
        let _ = output.write(symbols[(y & 0xff) as usize]);
    }
}

fn encode_mut<Bit: BitWidth, Msb: Bool>(
    symbols: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) {
    debug_assert_eq!(output.len(), encode_len::<Bit>(input.len()));
    let bit = Bit::VAL;
    let enc = enc(bit);
    let dec = dec(bit);
    let n = input.len() / enc;
    let bs = match bit {
        5 => 2,
        6 => 4,
        _ => 1,
    };
    vectorize(n, bs, |i| {
        let input = unsafe { chunk_unchecked(input, enc, i) };
        let output = unsafe { chunk_mut_unchecked(output, dec, i) };
        encode_block::<Bit, Msb>(symbols, input, output);
    });
    encode_block::<Bit, Msb>(symbols, &input[enc * n ..], &mut output[dec * n ..]);
}

// Fails if an input character does not translate to a symbol. The error is the
// lowest index of such character. The output is not written to.
fn decode_block<Bit: BitWidth, Msb: Bool>(
    values: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) -> Result<(), usize> {
    debug_assert!(output.len() <= enc(Bit::VAL));
    debug_assert_eq!(input.len(), encode_len::<Bit>(output.len()));
    let bit = Bit::VAL;
    let msb = Msb::VAL;
    let mut x = 0u64;
    for j in 0 .. input.len() {
        let y = values[input[j] as usize];
        check!(j, y < 1 << bit);
        x |= u64::from(y) << (bit * order(msb, dec(bit), j));
    }
    for (j, output) in output.iter_mut().enumerate() {
        let _ = output.write((x >> (8 * order(msb, enc(bit), j)) & 0xff) as u8);
    }
    Ok(())
}

// Fails if an input character does not translate to a symbol. The error `pos`
// is the lowest index of such character. The output is valid up to `pos / dec *
// enc` excluded.
fn decode_mut<Bit: BitWidth, Msb: Bool>(
    values: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) -> Result<(), usize> {
    debug_assert_eq!(input.len(), encode_len::<Bit>(output.len()));
    let bit = Bit::VAL;
    let enc = enc(bit);
    let dec = dec(bit);
    let n = input.len() / dec;
    for i in 0 .. n {
        let input = unsafe { chunk_unchecked(input, dec, i) };
        let output = unsafe { chunk_mut_unchecked(output, enc, i) };
        decode_block::<Bit, Msb>(values, input, output).map_err(|e| dec * i + e)?;
    }
    decode_block::<Bit, Msb>(values, &input[dec * n ..], &mut output[enc * n ..])
        .map_err(|e| dec * n + e)
}

// Fails if there are non-zero trailing bits.
fn check_trail<Bit: BitWidth, Msb: Bool>(
    ctb: bool, values: &[u8; 256], input: &[u8],
) -> Result<(), ()> {
    if 8 % Bit::VAL == 0 || !ctb {
        return Ok(());
    }
    let trail = Bit::VAL * input.len() % 8;
    if trail == 0 {
        return Ok(());
    }
    let mut mask = (1 << trail) - 1;
    if !Msb::VAL {
        mask <<= Bit::VAL - trail;
    }
    check!((), values[input[input.len() - 1] as usize] & mask == 0);
    Ok(())
}

// Fails if the padding length is invalid. The error is the index of the first
// padding character.
fn check_pad<Bit: BitWidth>(values: &[u8; 256], input: &[u8]) -> Result<usize, usize> {
    let bit = Bit::VAL;
    debug_assert_eq!(input.len(), dec(bit));
    let is_pad = |x: &&u8| values[**x as usize] == PADDING;
    let count = input.iter().rev().take_while(is_pad).count();
    let len = input.len() - count;
    check!(len, len > 0 && bit * len % 8 < bit);
    Ok(len)
}

fn encode_base_len<Bit: BitWidth>(len: usize) -> usize {
    encode_len::<Bit>(len)
}

fn encode_base<Bit: BitWidth, Msb: Bool>(
    symbols: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) {
    debug_assert_eq!(output.len(), encode_base_len::<Bit>(input.len()));
    encode_mut::<Bit, Msb>(symbols, input, output);
}

fn encode_pad_len<Bit: BitWidth, Pad: Bool>(len: usize) -> usize {
    match Pad::VAL {
        false => encode_base_len::<Bit>(len),
        true => div_ceil(len, enc(Bit::VAL)) * dec(Bit::VAL),
    }
}

fn encode_pad<Bit: BitWidth, Msb: Bool, Pad: Bool>(
    symbols: &[u8; 256], pad: Pad::If<u8, ()>, input: &[u8], output: &mut [MaybeUninit<u8>],
) {
    let pad = match Pad::open(pad) {
        If::Then(x) => x,
        If::Else(()) => return encode_base::<Bit, Msb>(symbols, input, output),
    };
    debug_assert_eq!(output.len(), encode_pad_len::<Bit, Pad>(input.len()));
    let olen = encode_base_len::<Bit>(input.len());
    encode_base::<Bit, Msb>(symbols, input, &mut output[.. olen]);
    for output in output.iter_mut().skip(olen) {
        let _ = output.write(pad);
    }
}

fn encode_wrap_len<Bit: BitWidth, Pad: Bool, Wrap: Bool>(
    wrap: Wrap::If<(usize, &[u8]), ()>, ilen: usize,
) -> usize {
    let olen = encode_pad_len::<Bit, Pad>(ilen);
    match Wrap::open(wrap) {
        If::Then((col, end)) => olen + end.len() * div_ceil(olen, col),
        If::Else(()) => olen,
    }
}

fn encode_wrap_mut<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool>(
    symbols: &[u8; 256], pad: Pad::If<u8, ()>, wrap: Wrap::If<(usize, &[u8]), ()>, input: &[u8],
    output: &mut [MaybeUninit<u8>],
) {
    let (col, end) = match Wrap::open(wrap) {
        If::Then((col, end)) => (col, end),
        If::Else(()) => return encode_pad::<Bit, Msb, Pad>(symbols, pad, input, output),
    };
    debug_assert_eq!(output.len(), encode_wrap_len::<Bit, Pad, Wrap>(wrap, input.len()));
    debug_assert_eq!(col % dec(Bit::VAL), 0);
    let bit = Bit::VAL;
    let col = col / dec(bit);
    let enc = col * enc(bit);
    let dec = col * dec(bit) + end.len();
    let olen = dec - end.len();
    let n = input.len() / enc;
    for i in 0 .. n {
        let input = unsafe { chunk_unchecked(input, enc, i) };
        let output = unsafe { chunk_mut_unchecked(output, dec, i) };
        encode_base::<Bit, Msb>(symbols, input, &mut output[.. olen]);
        unsafe { copy_from_slice(&mut output[olen ..], end) };
    }
    if input.len() > enc * n {
        let olen = dec * n + encode_pad_len::<Bit, Pad>(input.len() - enc * n);
        encode_pad::<Bit, Msb, Pad>(symbols, pad, &input[enc * n ..], &mut output[dec * n .. olen]);
        unsafe { copy_from_slice(&mut output[olen ..], end) };
    }
}

// Returns the longest valid input length and associated output length.
fn decode_wrap_len<Bit: BitWidth, Pad: Bool>(len: usize) -> (usize, usize) {
    let bit = Bit::VAL;
    if Pad::VAL {
        (floor(len, dec(bit)), len / dec(bit) * enc(bit))
    } else {
        let trail = bit * len % 8;
        (len - trail / bit, bit * len / 8)
    }
}

// Fails with Length if length is invalid. The error is the largest valid
// length.
fn decode_pad_len<Bit: BitWidth, Pad: Bool>(len: usize) -> Result<usize, DecodeError> {
    let (ilen, olen) = decode_wrap_len::<Bit, Pad>(len);
    check!(DecodeError { position: ilen, kind: DecodeKind::Length }, ilen == len);
    Ok(olen)
}

// Fails with Length if length is invalid. The error is the largest valid
// length.
fn decode_base_len<Bit: BitWidth>(len: usize) -> Result<usize, DecodeError> {
    decode_pad_len::<Bit, False>(len)
}

// Fails with Symbol if an input character does not translate to a symbol. The
// error is the lowest index of such character.
// Fails with Trailing if there are non-zero trailing bits.
fn decode_base_mut<Bit: BitWidth, Msb: Bool>(
    ctb: bool, values: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) -> Result<usize, DecodePartial> {
    debug_assert_eq!(Ok(output.len()), decode_base_len::<Bit>(input.len()));
    let bit = Bit::VAL;
    let fail = |pos, kind| DecodePartial {
        read: pos / dec(bit) * dec(bit),
        written: pos / dec(bit) * enc(bit),
        error: DecodeError { position: pos, kind },
    };
    decode_mut::<Bit, Msb>(values, input, output).map_err(|pos| fail(pos, DecodeKind::Symbol))?;
    check_trail::<Bit, Msb>(ctb, values, input)
        .map_err(|()| fail(input.len() - 1, DecodeKind::Trailing))?;
    Ok(output.len())
}

// Fails with Symbol if an input character does not translate to a symbol. The
// error is the lowest index of such character.
// Fails with Padding if some padding length is invalid. The error is the index
// of the first padding character of the invalid padding.
// Fails with Trailing if there are non-zero trailing bits.
fn decode_pad_mut<Bit: BitWidth, Msb: Bool, Pad: Bool>(
    ctb: bool, values: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) -> Result<usize, DecodePartial> {
    if !Pad::VAL {
        return decode_base_mut::<Bit, Msb>(ctb, values, input, output);
    }
    debug_assert_eq!(Ok(output.len()), decode_pad_len::<Bit, Pad>(input.len()));
    let bit = Bit::VAL;
    let enc = enc(bit);
    let dec = dec(bit);
    let mut inpos = 0;
    let mut outpos = 0;
    let mut outend = output.len();
    while inpos < input.len() {
        match decode_base_mut::<Bit, Msb>(
            ctb,
            values,
            &input[inpos ..],
            &mut output[outpos .. outend],
        ) {
            Ok(written) => {
                if cfg!(debug_assertions) {
                    inpos = input.len();
                }
                outpos += written;
                break;
            }
            Err(partial) => {
                inpos += partial.read;
                outpos += partial.written;
            }
        }
        let inlen = check_pad::<Bit>(values, &input[inpos .. inpos + dec]).map_err(|pos| {
            DecodePartial {
                read: inpos,
                written: outpos,
                error: DecodeError { position: inpos + pos, kind: DecodeKind::Padding },
            }
        })?;
        let outlen = decode_base_len::<Bit>(inlen).unwrap();
        let written = decode_base_mut::<Bit, Msb>(
            ctb,
            values,
            &input[inpos .. inpos + inlen],
            &mut output[outpos .. outpos + outlen],
        )
        .map_err(|partial| {
            debug_assert_eq!(partial.read, 0);
            debug_assert_eq!(partial.written, 0);
            DecodePartial {
                read: inpos,
                written: outpos,
                error: DecodeError {
                    position: inpos + partial.error.position,
                    kind: partial.error.kind,
                },
            }
        })?;
        debug_assert_eq!(written, outlen);
        inpos += dec;
        outpos += outlen;
        outend -= enc - outlen;
    }
    debug_assert_eq!(inpos, input.len());
    debug_assert_eq!(outpos, outend);
    Ok(outend)
}

fn skip_ignore(values: &[u8; 256], input: &[u8], mut inpos: usize) -> usize {
    while inpos < input.len() && values[input[inpos] as usize] == IGNORE {
        inpos += 1;
    }
    inpos
}

// Returns next input and output position.
// Fails with Symbol if an input character does not translate to a symbol. The
// error is the lowest index of such character.
// Fails with Padding if some padding length is invalid. The error is the index
// of the first padding character of the invalid padding.
// Fails with Trailing if there are non-zero trailing bits.
fn decode_wrap_block<Bit: BitWidth, Msb: Bool, Pad: Bool>(
    ctb: bool, values: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) -> Result<(usize, usize), DecodeError> {
    let bit = Bit::VAL;
    let dec = dec(bit);
    let mut buf = [0u8; 8];
    let mut shift = [0usize; 8];
    let mut bufpos = 0;
    let mut inpos = 0;
    while bufpos < dec {
        inpos = skip_ignore(values, input, inpos);
        if inpos == input.len() {
            break;
        }
        shift[bufpos] = inpos;
        buf[bufpos] = input[inpos];
        bufpos += 1;
        inpos += 1;
    }
    let olen = decode_pad_len::<Bit, Pad>(bufpos).map_err(|mut e| {
        e.position = shift[e.position];
        e
    })?;
    let written =
        decode_pad_mut::<Bit, Msb, Pad>(ctb, values, &buf[.. bufpos], &mut output[.. olen])
            .map_err(|partial| {
                debug_assert_eq!(partial.read, 0);
                debug_assert_eq!(partial.written, 0);
                DecodeError { position: shift[partial.error.position], kind: partial.error.kind }
            })?;
    Ok((inpos, written))
}

// Fails with Symbol if an input character does not translate to a symbol. The
// error is the lowest index of such character.
// Fails with Padding if some padding length is invalid. The error is the index
// of the first padding character of the invalid padding.
// Fails with Trailing if there are non-zero trailing bits.
// Fails with Length if input length (without ignored characters) is invalid.
fn decode_wrap_mut<Bit: BitWidth, Msb: Bool, Pad: Bool, Ignore: Bool>(
    ctb: bool, values: &[u8; 256], input: &[u8], output: &mut [MaybeUninit<u8>],
) -> Result<usize, DecodePartial> {
    if !Ignore::VAL {
        return decode_pad_mut::<Bit, Msb, Pad>(ctb, values, input, output);
    }
    debug_assert_eq!(output.len(), decode_wrap_len::<Bit, Pad>(input.len()).1);
    let mut inpos = 0;
    let mut outpos = 0;
    while inpos < input.len() {
        let (inlen, outlen) = decode_wrap_len::<Bit, Pad>(input.len() - inpos);
        match decode_pad_mut::<Bit, Msb, Pad>(
            ctb,
            values,
            &input[inpos .. inpos + inlen],
            &mut output[outpos .. outpos + outlen],
        ) {
            Ok(written) => {
                inpos += inlen;
                outpos += written;
                break;
            }
            Err(partial) => {
                inpos += partial.read;
                outpos += partial.written;
            }
        }
        let (ipos, opos) = decode_wrap_block::<Bit, Msb, Pad>(
            ctb,
            values,
            &input[inpos ..],
            &mut output[outpos ..],
        )
        .map_err(|mut error| {
            error.position += inpos;
            DecodePartial { read: inpos, written: outpos, error }
        })?;
        inpos += ipos;
        outpos += opos;
    }
    let inpos = skip_ignore(values, input, inpos);
    if inpos == input.len() {
        Ok(outpos)
    } else {
        Err(DecodePartial {
            read: inpos,
            written: outpos,
            error: DecodeError { position: inpos, kind: DecodeKind::Length },
        })
    }
}

/// Error converting from a dynamic encoding to a static one.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConvertError {
    /// Different bit-width.
    BitWidth,

    /// Different bit-order.
    BitOrder,

    /// Different padding.
    Padding,

    /// Different wrap.
    Wrap,

    /// Different ignore.
    Ignore,
}

/// Base-conversion encoding.
///
/// See [`Specification`] for technical details or how to define a new one.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Encoding<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> {
    // The config match the fields in data. In particular, we have the following properties:
    // - If Bit is Bit1, Bit2, or Bit4 then Pad is False.
    // - If Wrap is True, then Ignore is True.
    data: InternalEncoding,
    _type: PhantomData<(Bit, Msb, Pad, Wrap, Ignore)>,
}

impl<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> PartialEq
    for Encoding<Bit, Msb, Pad, Wrap, Ignore>
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> Eq
    for Encoding<Bit, Msb, Pad, Wrap, Ignore>
{
}

impl<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool>
    Encoding<Bit, Msb, Pad, Wrap, Ignore>
{
    fn sym(&self) -> &[u8; 256] {
        self.data[0 .. 256].try_into().unwrap()
    }

    fn val(&self) -> &[u8; 256] {
        self.data[256 .. 512].try_into().unwrap()
    }

    fn pad(&self) -> Pad::If<u8, ()> {
        Pad::make(|| self.data[512], || ())
    }

    fn ctb(&self) -> bool {
        self.data[513] & 0x10 != 0
    }

    fn wrap(&self) -> Wrap::If<(usize, &[u8]), ()> {
        Wrap::make(|| (self.data[514] as usize, &self.data[515 ..]), || ())
    }

    fn has_ignore(&self) -> bool {
        self.data.len() >= 515
    }

    /// Minimum number of input and output blocks when encoding.
    fn block_len(&self) -> (usize, usize) {
        let bit = Bit::VAL;
        match Wrap::open(self.wrap()) {
            If::Then((col, end)) => (col / dec(bit) * enc(bit), col + end.len()),
            If::Else(()) => (enc(bit), dec(bit)),
        }
    }

    /// Returns the encoded length of an input of length `len`.
    ///
    /// See [`Self::encode_mut()`] for when to use it.
    #[must_use]
    pub fn encode_len(&self, len: usize) -> usize {
        encode_wrap_len::<Bit, Pad, Wrap>(self.wrap(), len)
    }

    /// Encodes `input` in `output`.
    ///
    /// # Panics
    ///
    /// Panics if the `output` length does not match the result of [`Self::encode_len()`] for the
    /// `input` length.
    pub fn encode_mut_uninit<'a>(
        &self, input: &[u8], output: &'a mut [MaybeUninit<u8>],
    ) -> &'a mut [u8] {
        assert_eq!(output.len(), self.encode_len(input.len()));
        encode_wrap_mut::<Bit, Msb, Pad, Wrap>(self.sym(), self.pad(), self.wrap(), input, output);
        unsafe { slice_assume_init_mut(output) }
    }

    /// Encodes `input` in `output`.
    ///
    /// # Panics
    ///
    /// Panics if the `output` length does not match the result of [`Self::encode_len()`] for the
    /// `input` length.
    pub fn encode_mut(&self, input: &[u8], output: &mut [u8]) {
        let _ = self.encode_mut_uninit(input, unsafe { slice_uninit_mut(output) });
    }

    /// Appends the encoding of `input` to `output`.
    #[cfg(feature = "alloc")]
    pub fn encode_append(&self, input: &[u8], output: &mut String) {
        let output = unsafe { output.as_mut_vec() };
        let output_len = output.len();
        let len = self.encode_len(input.len());
        let _len = self.encode_mut_uninit(input, reserve_spare(output, len)).len();
        debug_assert_eq!(len, _len);
        unsafe { output.set_len(output_len + len) };
    }

    /// Returns an object to encode a fragmented input and append it to `output`.
    ///
    /// See the documentation of [`Encoder`] for more details and examples.
    #[cfg(feature = "alloc")]
    pub fn new_encoder<'a>(
        &'a self, output: &'a mut String,
    ) -> Encoder<'a, Bit, Msb, Pad, Wrap, Ignore> {
        Encoder::new(self, output)
    }

    /// Writes the encoding of `input` to `output` using a temporary `buffer`.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is shorter than 510 bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when writing to the output fails.
    pub fn encode_write_buffer_uninit(
        &self, input: &[u8], output: &mut impl core::fmt::Write, buffer: &mut [MaybeUninit<u8>],
    ) -> core::fmt::Result {
        assert!(510 <= buffer.len());
        let (enc, dec) = self.block_len();
        for input in input.chunks(buffer.len() / dec * enc) {
            let buffer = &mut buffer[.. self.encode_len(input.len())];
            let buffer = self.encode_mut_uninit(input, buffer);
            output.write_str(unsafe { core::str::from_utf8_unchecked(buffer) })?;
        }
        Ok(())
    }

    /// Writes the encoding of `input` to `output` using a temporary `buffer`.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is shorter than 510 bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when writing to the output fails.
    pub fn encode_write_buffer(
        &self, input: &[u8], output: &mut impl core::fmt::Write, buffer: &mut [u8],
    ) -> core::fmt::Result {
        self.encode_write_buffer_uninit(input, output, unsafe { slice_uninit_mut(buffer) })
    }

    /// Writes the encoding of `input` to `output`.
    ///
    /// This allocates a buffer of 1024 bytes on the stack. If you want to control the buffer size
    /// and location, use [`Self::encode_write_buffer()`] instead.
    ///
    /// # Errors
    ///
    /// Returns an error when writing to the output fails.
    pub fn encode_write(
        &self, input: &[u8], output: &mut impl core::fmt::Write,
    ) -> core::fmt::Result {
        self.encode_write_buffer(input, output, &mut [0; 1024])
    }

    /// Returns encoded `input`.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self, input: &[u8]) -> String {
        let mut output = Vec::new();
        let len = self.encode_len(input.len());
        let _len = self.encode_mut_uninit(input, reserve_spare(&mut output, len)).len();
        debug_assert_eq!(len, _len);
        unsafe { output.set_len(len) };
        unsafe { String::from_utf8_unchecked(output) }
    }

    /// Returns the decoded length of an input of length `len`.
    ///
    /// See [`Self::decode_mut()`] for when to use it.
    ///
    /// # Errors
    ///
    /// Returns an error if `len` is invalid. The error [kind][DecodeError::kind] is
    /// [`DecodeKind::Length`] and the error [position][DecodeError::position] is the greatest valid
    /// input length.
    pub fn decode_len(&self, len: usize) -> Result<usize, DecodeError> {
        let (ilen, olen) = decode_wrap_len::<Bit, Pad>(len);
        check!(
            DecodeError { position: ilen, kind: DecodeKind::Length },
            self.has_ignore() || len == ilen
        );
        Ok(olen)
    }

    /// Decodes `input` in `output`.
    ///
    /// Returns the decoded output. Its length may be smaller than the output length if the input
    /// contained padding or ignored characters. The output bytes after the returned length are not
    /// initialized and should not be read.
    ///
    /// # Panics
    ///
    /// Panics if the `output` length does not match the result of [`Self::decode_len()`] for the
    /// `input` length. Also panics if `decode_len` fails for the `input` length.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. See [`Self::decode_mut()`] for more details.
    pub fn decode_mut_uninit<'a>(
        &self, input: &[u8], output: &'a mut [MaybeUninit<u8>],
    ) -> Result<&'a mut [u8], DecodePartial> {
        assert_eq!(Ok(output.len()), self.decode_len(input.len()));
        let len = decode_wrap_mut::<Bit, Msb, Pad, Ignore>(self.ctb(), self.val(), input, output)?;
        Ok(unsafe { slice_assume_init_mut(&mut output[.. len]) })
    }

    /// Decodes `input` in `output`.
    ///
    /// Returns the length of the decoded output. This length may be smaller than the output length
    /// if the input contained padding or ignored characters. The output bytes after the returned
    /// length are not initialized and should not be read.
    ///
    /// # Panics
    ///
    /// Panics if the `output` length does not match the result of [`Self::decode_len()`] for the
    /// `input` length. Also panics if `decode_len` fails for the `input` length.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. See [`Self::decode()`] for more details. The are two
    /// differences though:
    ///
    /// - [`DecodeKind::Length`] may be returned only if the encoding allows ignored characters,
    ///   because otherwise this is already checked by [`Self::decode_len()`].
    /// - The [`DecodePartial::read`] first bytes of the input have been successfully decoded to the
    ///   [`DecodePartial::written`] first bytes of the output.
    pub fn decode_mut(&self, input: &[u8], output: &mut [u8]) -> Result<usize, DecodePartial> {
        Ok(self.decode_mut_uninit(input, unsafe { slice_uninit_mut(output) })?.len())
    }

    /// Returns decoded `input`.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be:
    ///
    /// - [`DecodeKind::Length`] if the input length is invalid. The [position] is the greatest
    ///   valid input length.
    /// - [`DecodeKind::Symbol`] if the input contains an invalid character. The [position] is the
    ///   first invalid character.
    /// - [`DecodeKind::Trailing`] if the input has non-zero trailing bits. This is only possible if
    ///   the encoding checks trailing bits. The [position] is the first character containing
    ///   non-zero trailing bits.
    /// - [`DecodeKind::Padding`] if the input has an invalid padding length. This is only possible
    ///   if the encoding uses padding. The [position] is the first padding character of the first
    ///   padding of invalid length.
    ///
    /// [position]: DecodeError::position
    #[cfg(feature = "alloc")]
    pub fn decode(&self, input: &[u8]) -> Result<Vec<u8>, DecodeError> {
        let max_len = self.decode_len(input.len())?;
        let mut output = Vec::new();
        let len = self
            .decode_mut_uninit(input, reserve_spare(&mut output, max_len))
            .map_err(|partial| partial.error)?
            .len();
        unsafe { output.set_len(len) };
        Ok(output)
    }

    #[doc(hidden)]
    #[must_use]
    pub const unsafe fn new_unchecked(data: &'static [u8]) -> Self {
        #[cfg(feature = "alloc")]
        let data = Cow::Borrowed(data);
        Encoding { data, _type: PhantomData }
    }

    fn check_compatible(base: &DynEncoding) -> Result<(), ConvertError> {
        check!(ConvertError::BitWidth, base.bit() == Bit::VAL);
        check!(ConvertError::BitOrder, base.msb() == Msb::VAL);
        check!(ConvertError::Padding, base.pad().is_some() == Pad::VAL);
        check!(ConvertError::Wrap, base.wrap().is_some() == Wrap::VAL);
        check!(ConvertError::Ignore, base.has_ignore() == Ignore::VAL);
        Ok(())
    }
}

/// Encodes fragmented input to an output.
///
/// It is equivalent to use an [`Encoder`] with multiple calls to [`Encoder::append()`] than to
/// first concatenate all the input and then use [`Encoding::encode_append()`]. In particular, this
/// function will not introduce padding or wrapping between inputs.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct Encoder<'a, Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> {
    encoding: &'a Encoding<Bit, Msb, Pad, Wrap, Ignore>,
    output: &'a mut String,
    buffer: [u8; 255],
    length: u8,
}

#[cfg(feature = "alloc")]
impl<'a, Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> Drop
    for Encoder<'a, Bit, Msb, Pad, Wrap, Ignore>
{
    fn drop(&mut self) {
        self.encoding.encode_append(&self.buffer[.. self.length as usize], self.output);
    }
}

#[cfg(feature = "alloc")]
impl<'a, Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool>
    Encoder<'a, Bit, Msb, Pad, Wrap, Ignore>
{
    fn new(encoding: &'a Encoding<Bit, Msb, Pad, Wrap, Ignore>, output: &'a mut String) -> Self {
        Encoder { encoding, output, buffer: [0; 255], length: 0 }
    }

    /// Encodes the provided input fragment and appends the result to the output.
    pub fn append(&mut self, mut input: &[u8]) {
        #[allow(clippy::cast_possible_truncation)] // no truncation
        let max = self.encoding.block_len().0 as u8;
        if self.length != 0 {
            let len = self.length;
            #[allow(clippy::cast_possible_truncation)] // no truncation
            let add = core::cmp::min((max - len) as usize, input.len()) as u8;
            self.buffer[len as usize ..][.. add as usize].copy_from_slice(&input[.. add as usize]);
            self.length += add;
            input = &input[add as usize ..];
            if self.length != max {
                debug_assert!(self.length < max);
                debug_assert!(input.is_empty());
                return;
            }
            self.encoding.encode_append(&self.buffer[.. max as usize], self.output);
            self.length = 0;
        }
        let len = floor(input.len(), max as usize);
        self.encoding.encode_append(&input[.. len], self.output);
        input = &input[len ..];
        #[allow(clippy::cast_possible_truncation)] // no truncation
        let len = input.len() as u8;
        self.buffer[.. len as usize].copy_from_slice(input);
        self.length = len;
    }

    /// Makes sure all inputs have been encoded and appended to the output.
    ///
    /// This is equivalent to dropping the encoder and required for correctness, otherwise some
    /// encoded data may be missing at the end.
    pub fn finalize(self) {}
}

impl<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> TryFrom<DynEncoding>
    for Encoding<Bit, Msb, Pad, Wrap, Ignore>
{
    type Error = ConvertError;

    fn try_from(base: DynEncoding) -> Result<Self, Self::Error> {
        Encoding::<Bit, Msb, Pad, Wrap, Ignore>::check_compatible(&base)?;
        Ok(Encoding { data: base.0, _type: PhantomData })
    }
}

impl<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool>
    From<Encoding<Bit, Msb, Pad, Wrap, Ignore>> for DynEncoding
{
    fn from(base: Encoding<Bit, Msb, Pad, Wrap, Ignore>) -> Self {
        DynEncoding(base.data)
    }
}

impl<'a, Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> TryFrom<&'a DynEncoding>
    for &'a Encoding<Bit, Msb, Pad, Wrap, Ignore>
{
    type Error = ConvertError;

    fn try_from(base: &'a DynEncoding) -> Result<Self, Self::Error> {
        Encoding::<Bit, Msb, Pad, Wrap, Ignore>::check_compatible(base)?;
        Ok(unsafe {
            &*(base as *const DynEncoding).cast::<Encoding<Bit, Msb, Pad, Wrap, Ignore>>()
        })
    }
}

impl<'a, Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool>
    From<&'a Encoding<Bit, Msb, Pad, Wrap, Ignore>> for &'a DynEncoding
{
    fn from(base: &'a Encoding<Bit, Msb, Pad, Wrap, Ignore>) -> Self {
        unsafe { &*(base as *const Encoding<Bit, Msb, Pad, Wrap, Ignore>).cast::<DynEncoding>() }
    }
}

/// Hexadecimal encoding.
pub type Hex = Encoding<Bit4, True, False, False, False>;

/// Base32 encoding.
pub type Base32 = Encoding<Bit5, True, True, False, False>;

/// Base32 encoding (no padding).
pub type Base32NoPad = Encoding<Bit5, True, False, False, False>;

/// Base32 encoding (LSB first, no padding).
pub type Base32LsbNoPad = Encoding<Bit5, False, False, False, False>;

/// Base64 encoding.
pub type Base64 = Encoding<Bit6, True, True, False, False>;

/// Base64 encoding (no padding).
pub type Base64NoPad = Encoding<Bit6, True, False, False, False>;

/// Base64 encoding (wrap).
pub type Base64Wrap = Encoding<Bit6, True, True, True, True>;

/// Lowercase hexadecimal encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Hex, HEXLOWER};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789abcdef");
/// assert_eq!(HEXLOWER, Hex::try_from(spec.encoding().unwrap()).unwrap());
/// ```
pub static HEXLOWER: Hex = unsafe { Hex::new_unchecked(crate::HEXLOWER_IMPL) };

/// Lowercase hexadecimal encoding with case-insensitive decoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Hex, HEXLOWER_PERMISSIVE};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789abcdef");
/// spec.translate.from.push_str("ABCDEF");
/// spec.translate.to.push_str("abcdef");
/// assert_eq!(HEXLOWER_PERMISSIVE, Hex::try_from(spec.encoding().unwrap()).unwrap());
/// ```
pub static HEXLOWER_PERMISSIVE: Hex =
    unsafe { Hex::new_unchecked(crate::HEXLOWER_PERMISSIVE_IMPL) };

/// Uppercase hexadecimal encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Hex, HEXUPPER};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEF");
/// assert_eq!(HEXUPPER, Hex::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It is compliant with [RFC4648] and known as "base16" or "hex".
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-8
pub static HEXUPPER: Hex = unsafe { Hex::new_unchecked(crate::HEXUPPER_IMPL) };

/// Uppercase hexadecimal encoding with case-insensitive decoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Hex, HEXUPPER_PERMISSIVE};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEF");
/// spec.translate.from.push_str("abcdef");
/// spec.translate.to.push_str("ABCDEF");
/// assert_eq!(HEXUPPER_PERMISSIVE, Hex::try_from(spec.encoding().unwrap()).unwrap());
/// ```
pub static HEXUPPER_PERMISSIVE: Hex =
    unsafe { Hex::new_unchecked(crate::HEXUPPER_PERMISSIVE_IMPL) };

/// Padded base32 encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base32, BASE32};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");
/// spec.padding = Some('=');
/// assert_eq!(BASE32, Base32::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-6
pub static BASE32: Base32 = unsafe { Base32::new_unchecked(crate::BASE32_IMPL) };

/// Unpadded base32 encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base32NoPad, BASE32_NOPAD};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");
/// assert_eq!(BASE32_NOPAD, Base32NoPad::try_from(spec.encoding().unwrap()).unwrap());
/// ```
pub static BASE32_NOPAD: Base32NoPad =
    unsafe { Base32NoPad::new_unchecked(crate::BASE32_NOPAD_IMPL) };

/// Padded base32hex encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base32, BASE32HEX};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEFGHIJKLMNOPQRSTUV");
/// spec.padding = Some('=');
/// assert_eq!(BASE32HEX, Base32::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-7
pub static BASE32HEX: Base32 = unsafe { Base32::new_unchecked(crate::BASE32HEX_IMPL) };

/// Unpadded base32hex encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base32NoPad, BASE32HEX_NOPAD};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEFGHIJKLMNOPQRSTUV");
/// assert_eq!(BASE32HEX_NOPAD, Base32NoPad::try_from(spec.encoding().unwrap()).unwrap());
/// ```
pub static BASE32HEX_NOPAD: Base32NoPad =
    unsafe { Base32NoPad::new_unchecked(crate::BASE32HEX_NOPAD_IMPL) };

/// DNSSEC base32 encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base32NoPad, BASE32_DNSSEC};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789abcdefghijklmnopqrstuv");
/// spec.translate.from.push_str("ABCDEFGHIJKLMNOPQRSTUV");
/// spec.translate.to.push_str("abcdefghijklmnopqrstuv");
/// assert_eq!(BASE32_DNSSEC, Base32NoPad::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It conforms to [RFC5155]:
///
/// - It uses a base32 extended hex alphabet.
/// - It is case-insensitive when decoding and uses lowercase when encoding.
/// - It does not use padding.
///
/// [RFC5155]: https://tools.ietf.org/html/rfc5155
pub static BASE32_DNSSEC: Base32NoPad =
    unsafe { Base32NoPad::new_unchecked(crate::BASE32_DNSSEC_IMPL) };

#[allow(clippy::doc_markdown)]
/// DNSCurve base32 encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::{BitOrder, Specification};
/// # use data_encoding::v3_preview::{Base32LsbNoPad, BASE32_DNSCURVE};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789bcdfghjklmnpqrstuvwxyz");
/// spec.bit_order = BitOrder::LeastSignificantFirst;
/// spec.translate.from.push_str("BCDFGHJKLMNPQRSTUVWXYZ");
/// spec.translate.to.push_str("bcdfghjklmnpqrstuvwxyz");
/// assert_eq!(BASE32_DNSCURVE, Base32LsbNoPad::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It conforms to [DNSCurve].
///
/// [DNSCurve]: https://dnscurve.org/in-implement.html
pub static BASE32_DNSCURVE: Base32LsbNoPad =
    unsafe { Base32LsbNoPad::new_unchecked(crate::BASE32_DNSCURVE_IMPL) };

/// Padded base64 encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base64, BASE64};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// spec.padding = Some('=');
/// assert_eq!(BASE64, Base64::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-4
pub static BASE64: Base64 = unsafe { Base64::new_unchecked(crate::BASE64_IMPL) };

/// Unpadded base64 encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base64NoPad, BASE64_NOPAD};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// assert_eq!(BASE64_NOPAD, Base64NoPad::try_from(spec.encoding().unwrap()).unwrap());
/// ```
pub static BASE64_NOPAD: Base64NoPad =
    unsafe { Base64NoPad::new_unchecked(crate::BASE64_NOPAD_IMPL) };

/// MIME base64 encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base64Wrap, BASE64_MIME};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// spec.padding = Some('=');
/// spec.wrap.width = 76;
/// spec.wrap.separator.push_str("\r\n");
/// assert_eq!(BASE64_MIME, Base64Wrap::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It does not exactly conform to [RFC2045] because it does not print the header
/// and does not ignore all characters.
///
/// [RFC2045]: https://tools.ietf.org/html/rfc2045
pub static BASE64_MIME: Base64Wrap = unsafe { Base64Wrap::new_unchecked(crate::BASE64_MIME_IMPL) };

/// MIME base64 encoding without trailing bits check.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base64Wrap, BASE64_MIME_PERMISSIVE};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// spec.padding = Some('=');
/// spec.wrap.width = 76;
/// spec.wrap.separator.push_str("\r\n");
/// spec.check_trailing_bits = false;
/// assert_eq!(BASE64_MIME_PERMISSIVE, Base64Wrap::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It does not exactly conform to [RFC2045] because it does not print the header
/// and does not ignore all characters.
///
/// [RFC2045]: https://tools.ietf.org/html/rfc2045
pub static BASE64_MIME_PERMISSIVE: Base64Wrap =
    unsafe { Base64Wrap::new_unchecked(crate::BASE64_MIME_PERMISSIVE_IMPL) };

/// Padded base64url encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base64, BASE64URL};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_");
/// spec.padding = Some('=');
/// assert_eq!(BASE64URL, Base64::try_from(spec.encoding().unwrap()).unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-5
pub static BASE64URL: Base64 = unsafe { Base64::new_unchecked(crate::BASE64URL_IMPL) };

/// Unpadded base64url encoding.
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding::Specification;
/// # use data_encoding::v3_preview::{Base64NoPad, BASE64URL_NOPAD};
/// # use core::convert::TryFrom as _;
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_");
/// assert_eq!(BASE64URL_NOPAD, Base64NoPad::try_from(spec.encoding().unwrap()).unwrap());
/// ```
pub static BASE64URL_NOPAD: Base64NoPad =
    unsafe { Base64NoPad::new_unchecked(crate::BASE64URL_NOPAD_IMPL) };
