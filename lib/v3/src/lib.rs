//! Efficient and customizable data-encoding functions like base64, base32, and hex
#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
use alloc::borrow::{Cow, ToOwned};
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

macro_rules! check {
    ($e: expr, $c: expr) => {
        if !$c {
            return Err($e);
        }
    };
}

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

unsafe fn cast<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool>(
    base: &DynEncoding,
) -> &Encoding<Bit, Msb, Pad, Wrap, Ignore> {
    let ptr = core::ptr::from_ref(base).cast::<Encoding<Bit, Msb, Pad, Wrap, Ignore>>();
    unsafe { &*ptr }
}

macro_rules! dispatch {
    ($dyn:ident $($body: tt)*) => {
        dispatch!([] Bit $dyn $($body)*)
    };
    ([] Bit $dyn:ident $($body:tt)*) => {
        match $dyn.bit() {
            1 => dispatch!([Bit1] Msb $dyn $($body)*),
            2 => dispatch!([Bit2] Msb $dyn $($body)*),
            3 => dispatch!([Bit3] Msb $dyn $($body)*),
            4 => dispatch!([Bit4] Msb $dyn $($body)*),
            5 => dispatch!([Bit5] Msb $dyn $($body)*),
            6 => dispatch!([Bit6] Msb $dyn $($body)*),
            _ => unreachable!(),
        }
    };
    ([$($gen:ty),*] Msb $dyn:ident $($body:tt)*) => {
        match $dyn.msb() {
            false => dispatch!([$($gen),*, False] Pad $dyn $($body)*),
            true => dispatch!([$($gen),*, True] Pad $dyn $($body)*),
        }
    };
    ([$($gen:ty),*] Pad $dyn:ident $($body:tt)*) => {
        match $dyn.pad().is_some() {
            false => dispatch!([$($gen),*, False] Wrap $dyn $($body)*),
            true => dispatch!([$($gen),*, True] Wrap $dyn $($body)*),
        }
    };
    ([$($gen:ty),*] Wrap $dyn:ident $($body:tt)*) => {
        match $dyn.wrap().is_some() {
            false => dispatch!([$($gen),*, False] Ignore $dyn $($body)*),
            true => dispatch!([$($gen),*, True] Ignore $dyn $($body)*),
        }
    };
    ([$($gen:ty),*] Ignore $dyn:ident $($body:tt)*) => {
        match $dyn.has_ignore() {
            false => dispatch!({ $($gen),*, False } $dyn $($body)*),
            true => dispatch!({ $($gen),*, True } $dyn $($body)*),
        }
    };
    ({ $($gen:ty),* } $dyn:ident $($body:tt)*) => {
        unsafe { cast::<$($gen),*>($dyn) } $($body)*
    };
}

unsafe fn chunk_unchecked<T>(x: &[T], n: usize, i: usize) -> &[T] {
    debug_assert!((i + 1) * n <= x.len());
    unsafe { core::slice::from_raw_parts(x.as_ptr().add(n * i), n) }
}

unsafe fn chunk_mut_unchecked<T>(x: &mut [T], n: usize, i: usize) -> &mut [T] {
    debug_assert!((i + 1) * n <= x.len());
    unsafe { core::slice::from_raw_parts_mut(x.as_mut_ptr().add(n * i), n) }
}

// TODO(https://github.com/rust-lang/rust/issues/79995): Use write_slice() instead.
unsafe fn copy_from_slice(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
    dst.copy_from_slice(unsafe { &*(core::ptr::from_ref(src) as *const [MaybeUninit<u8>]) });
}

// TODO(https://github.com/rust-lang/rust/issues/63569): Use slice_assume_init_mut() instead.
unsafe fn slice_assume_init_mut(xs: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    unsafe { &mut *(core::ptr::from_mut(xs) as *mut [u8]) }
}

unsafe fn slice_uninit_mut(xs: &mut [u8]) -> &mut [MaybeUninit<u8>] {
    unsafe { &mut *(core::ptr::from_mut(xs) as *mut [MaybeUninit<u8>]) }
}

#[cfg(feature = "alloc")]
fn reserve_spare(xs: &mut Vec<u8>, n: usize) -> &mut [MaybeUninit<u8>] {
    xs.reserve(n);
    &mut xs.spare_capacity_mut()[.. n]
}

fn floor(x: usize, m: usize) -> usize {
    x / m * m
}

#[inline]
fn vectorize<F: FnMut(usize)>(n: usize, bs: usize, mut f: F) {
    for k in 0 .. n / bs {
        for i in k * bs .. (k + 1) * bs {
            f(i);
        }
    }
    for i in floor(n, bs) .. n {
        f(i);
    }
}

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

impl core::fmt::Display for DecodeKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let description = match self {
            DecodeKind::Length => "invalid length",
            DecodeKind::Symbol => "invalid symbol",
            DecodeKind::Trailing => "non-zero trailing bits",
            DecodeKind::Padding => "invalid padding length",
        };
        write!(f, "{description}")
    }
}

/// Decoding error
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DecodeError {
    /// Error position
    ///
    /// This position is always a valid input position and represents the first encountered error.
    pub position: usize,

    /// Error kind
    pub kind: DecodeKind,
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} at {}", self.kind, self.position)
    }
}

/// Decoding error with partial result
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DecodePartial {
    /// Number of bytes read from input
    ///
    /// This number does not exceed the error position: `read <= error.position`.
    pub read: usize,

    /// Number of bytes written to output
    ///
    /// This number does not exceed the decoded length: `written <= decode_len(read)`.
    pub written: usize,

    /// Decoding error
    pub error: DecodeError,
}

const INVALID: u8 = 128;
const IGNORE: u8 = 129;
const PADDING: u8 = 130;

fn order(msb: bool, n: usize, i: usize) -> usize {
    if msb { n - 1 - i } else { i }
}

#[inline]
fn enc(bit: usize) -> usize {
    match bit {
        1 | 2 | 4 => 1,
        3 | 6 => 3,
        5 => 5,
        _ => unreachable!(),
    }
}

#[inline]
fn dec(bit: usize) -> usize {
    enc(bit) * 8 / bit
}

fn encode_len<Bit: BitWidth>(len: usize) -> usize {
    (8 * len).div_ceil(Bit::VAL)
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
        true => len.div_ceil(enc(Bit::VAL)) * dec(Bit::VAL),
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
        If::Then((col, end)) => olen + end.len() * olen.div_ceil(col),
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Encoding<Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool> {
    // The config match the fields in data. In particular, we have the following properties:
    // - If Bit is Bit1, Bit2, or Bit4 then Pad is False.
    // - If Wrap is True, then Ignore is True.
    data: InternalEncoding,
    _type: PhantomData<(Bit, Msb, Pad, Wrap, Ignore)>,
}

/// Base-conversion encoding
///
/// See [`Specification`] for technical details or how to define a new one.
// Required fields:
//   0 - 256 (256) symbols
// 256 - 512 (256) values
// 512 - 513 (  1) padding
// 513 - 514 (  1) reserved(3),ctb(1),msb(1),bit(3)
// Optional fields:
// 514 - 515 (  1) width
// 515 -   * (  N) separator
// Invariants:
// - symbols is 2^bit unique characters repeated 2^(8-bit) times
// - values[128 ..] are INVALID
// - values[0 .. 128] are either INVALID, IGNORE, PADDING, or < 2^bit
// - padding is either < 128 or INVALID
// - values[padding] is PADDING if padding < 128
// - values and symbols are inverse
// - ctb is true if 8 % bit == 0
// - width is present if there is x such that values[x] is IGNORE
// - width % dec(bit) == 0
// - for all x in separator values[x] is IGNORE
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct DynEncoding(InternalEncoding);

#[cfg(feature = "alloc")]
type InternalEncoding = Cow<'static, [u8]>;

#[cfg(not(feature = "alloc"))]
type InternalEncoding = &'static [u8];

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
        let actual_len = self.encode_mut_uninit(input, reserve_spare(output, len)).len();
        debug_assert_eq!(actual_len, len);
        unsafe { output.set_len(output_len + len) };
    }

    // /// Returns an object to encode a fragmented input and append it to `output`.
    // ///
    // /// See the documentation of [`Encoder`] for more details and examples.
    // #[cfg(feature = "alloc")]
    // pub fn new_encoder<'a>(
    //     &'a self, output: &'a mut String,
    // ) -> Encoder<'a, Bit, Msb, Pad, Wrap, Ignore> {
    //     Encoder::new(self, output)
    // }

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
        let actual_len = self.encode_mut_uninit(input, reserve_spare(&mut output, len)).len();
        debug_assert_eq!(actual_len, len);
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

    /// TODO
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn specification(&self) -> Specification {
        DynEncoding::specification(self.into())
    }

    /// TODO
    #[must_use]
    pub fn as_dyn(&self) -> &DynEncoding {
        self.into()
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

impl DynEncoding {
    fn sym(&self) -> &[u8; 256] {
        self.0[0 .. 256].try_into().unwrap()
    }

    fn val(&self) -> &[u8; 256] {
        self.0[256 .. 512].try_into().unwrap()
    }

    fn pad(&self) -> Option<u8> {
        if self.0[512] < 128 { Some(self.0[512]) } else { None }
    }

    fn ctb(&self) -> bool {
        self.0[513] & 0x10 != 0
    }

    fn msb(&self) -> bool {
        self.0[513] & 0x8 != 0
    }

    fn bit(&self) -> usize {
        (self.0[513] & 0x7) as usize
    }

    /// Minimum number of input and output blocks when encoding
    fn block_len(&self) -> (usize, usize) {
        let bit = self.bit();
        match self.wrap() {
            Some((col, end)) => (col / dec(bit) * enc(bit), col + end.len()),
            None => (enc(bit), dec(bit)),
        }
    }

    fn wrap(&self) -> Option<(usize, &[u8])> {
        if self.0.len() <= 515 {
            return None;
        }
        Some((self.0[514] as usize, &self.0[515 ..]))
    }

    fn has_ignore(&self) -> bool {
        self.0.len() >= 515
    }

    /// Returns the encoded length of an input of length `len`
    ///
    /// See [`encode_mut`] for when to use it.
    ///
    /// [`encode_mut`]: struct.Encoding.html#method.encode_mut
    #[must_use]
    pub fn encode_len(&self, len: usize) -> usize {
        dispatch!(self.encode_len(len))
    }

    /// Encodes `input` in `output`
    ///
    /// # Panics
    ///
    /// Panics if the `output` length does not match the result of [`encode_len`] for the `input`
    /// length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding_v3::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// let input = b"Hello world";
    /// let output = &mut buffer[0 .. BASE64.encode_len(input.len())];
    /// BASE64.encode_mut(input, output);
    /// assert_eq!(output, b"SGVsbG8gd29ybGQ=");
    /// ```
    ///
    /// [`encode_len`]: struct.Encoding.html#method.encode_len
    #[allow(clippy::cognitive_complexity)]
    pub fn encode_mut(&self, input: &[u8], output: &mut [u8]) {
        dispatch!(self.encode_mut(input, output))
    }

    /// Appends the encoding of `input` to `output`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding_v3::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// let input = b"Hello world";
    /// let mut output = "Result: ".to_string();
    /// BASE64.encode_append(input, &mut output);
    /// assert_eq!(output, "Result: SGVsbG8gd29ybGQ=");
    /// ```
    #[cfg(feature = "alloc")]
    pub fn encode_append(&self, input: &[u8], output: &mut String) {
        let output = unsafe { output.as_mut_vec() };
        let output_len = output.len();
        output.resize(output_len + self.encode_len(input.len()), 0u8);
        self.encode_mut(input, &mut output[output_len ..]);
    }

    /// Writes the encoding of `input` to `output`
    ///
    /// This allocates a buffer of 1024 bytes on the stack. If you want to control the buffer size
    /// and location, use [`Encoding::encode_write_buffer()`] instead.
    ///
    /// # Errors
    ///
    /// Returns an error when writing to the output fails.
    pub fn encode_write(
        &self, input: &[u8], output: &mut impl core::fmt::Write,
    ) -> core::fmt::Result {
        self.encode_write_buffer(input, output, &mut [0; 1024])
    }

    /// Writes the encoding of `input` to `output` using a temporary `buffer`
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
        assert!(510 <= buffer.len());
        let (enc, dec) = self.block_len();
        for input in input.chunks(buffer.len() / dec * enc) {
            let buffer = &mut buffer[.. self.encode_len(input.len())];
            self.encode_mut(input, buffer);
            output.write_str(unsafe { core::str::from_utf8_unchecked(buffer) })?;
        }
        Ok(())
    }

    /// Returns encoded `input`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding_v3::BASE64;
    /// assert_eq!(BASE64.encode(b"Hello world"), "SGVsbG8gd29ybGQ=");
    /// ```
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self, input: &[u8]) -> String {
        let mut output = vec![0u8; self.encode_len(input.len())];
        self.encode_mut(input, &mut output);
        unsafe { String::from_utf8_unchecked(output) }
    }

    /// Returns the maximum decoded length of an input of length `len`
    ///
    /// See [`decode_mut`] for when to use it. In particular, the actual decoded length might be
    /// smaller if the actual input contains padding or ignored characters.
    ///
    /// # Errors
    ///
    /// Returns an error if `len` is invalid. The error kind is [`Length`] and the [position] is the
    /// greatest valid input length.
    ///
    /// [`decode_mut`]: struct.Encoding.html#method.decode_mut
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [position]: struct.DecodeError.html#structfield.position
    pub fn decode_len(&self, len: usize) -> Result<usize, DecodeError> {
        dispatch!(self.decode_len(len))
    }

    /// Decodes `input` in `output`
    ///
    /// Returns the length of the decoded output. This length may be smaller than the output length
    /// if the input contained padding or ignored characters. The output bytes after the returned
    /// length are not initialized and should not be read.
    ///
    /// # Panics
    ///
    /// Panics if the `output` length does not match the result of [`decode_len`] for the `input`
    /// length. Also panics if `decode_len` fails for the `input` length.
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. See [`decode`] for more details. The are two
    /// differences though:
    ///
    /// - [`Length`] may be returned only if the encoding allows ignored characters, because
    ///   otherwise this is already checked by [`decode_len`].
    /// - The [`read`] first bytes of the input have been successfully decoded to the [`written`]
    ///   first bytes of the output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding_v3::BASE64;
    /// # let mut buffer = vec![0; 100];
    /// let input = b"SGVsbA==byB3b3JsZA==";
    /// let output = &mut buffer[0 .. BASE64.decode_len(input.len()).unwrap()];
    /// let len = BASE64.decode_mut(input, output).unwrap();
    /// assert_eq!(&output[0 .. len], b"Hello world");
    /// ```
    ///
    /// [`decode_len`]: struct.Encoding.html#method.decode_len
    /// [`decode`]: struct.Encoding.html#method.decode
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [`read`]: struct.DecodePartial.html#structfield.read
    /// [`written`]: struct.DecodePartial.html#structfield.written
    #[allow(clippy::cognitive_complexity)]
    pub fn decode_mut(&self, input: &[u8], output: &mut [u8]) -> Result<usize, DecodePartial> {
        dispatch!(self.decode_mut(input, output))
    }

    /// Returns decoded `input`
    ///
    /// # Errors
    ///
    /// Returns an error if `input` is invalid. The error kind can be:
    ///
    /// - [`Length`] if the input length is invalid. The [position] is the greatest valid input
    ///   length.
    /// - [`Symbol`] if the input contains an invalid character. The [position] is the first invalid
    ///   character.
    /// - [`Trailing`] if the input has non-zero trailing bits. This is only possible if the
    ///   encoding checks trailing bits. The [position] is the first character containing non-zero
    ///   trailing bits.
    /// - [`Padding`] if the input has an invalid padding length. This is only possible if the
    ///   encoding uses padding. The [position] is the first padding character of the first padding
    ///   of invalid length.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use data_encoding_v3::BASE64;
    /// assert_eq!(BASE64.decode(b"SGVsbA==byB3b3JsZA==").unwrap(), b"Hello world");
    /// ```
    ///
    /// [`Length`]: enum.DecodeKind.html#variant.Length
    /// [`Symbol`]: enum.DecodeKind.html#variant.Symbol
    /// [`Trailing`]: enum.DecodeKind.html#variant.Trailing
    /// [`Padding`]: enum.DecodeKind.html#variant.Padding
    /// [position]: struct.DecodeError.html#structfield.position
    #[cfg(feature = "alloc")]
    pub fn decode(&self, input: &[u8]) -> Result<Vec<u8>, DecodeError> {
        let mut output = vec![0u8; self.decode_len(input.len())?];
        let len = self.decode_mut(input, &mut output).map_err(|partial| partial.error)?;
        output.truncate(len);
        Ok(output)
    }

    /// Returns the bit-width
    #[must_use]
    pub fn bit_width(&self) -> usize {
        self.bit()
    }

    /// Returns whether the encoding is canonical
    ///
    /// An encoding is not canonical if one of the following conditions holds:
    ///
    /// - trailing bits are not checked
    /// - padding is used
    /// - characters are ignored
    /// - characters are translated
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        if !self.ctb() {
            return false;
        }
        let bit = self.bit();
        let sym = self.sym();
        let val = self.val();
        for i in 0 .. 256 {
            if val[i] == INVALID {
                continue;
            }
            if val[i] >= 1 << bit {
                return false;
            }
            if sym[val[i] as usize] as usize != i {
                return false;
            }
        }
        true
    }

    /// Returns the encoding specification
    #[allow(clippy::missing_panics_doc)] // no panic
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn specification(&self) -> Specification {
        let mut specification = Specification::new();
        specification
            .symbols
            .push_str(core::str::from_utf8(&self.sym()[0 .. 1 << self.bit()]).unwrap());
        specification.bit_order =
            if self.msb() { MostSignificantFirst } else { LeastSignificantFirst };
        specification.check_trailing_bits = self.ctb();
        if let Some(pad) = self.pad() {
            specification.padding = Some(pad as char);
        }
        for i in 0 .. 128u8 {
            if self.val()[i as usize] != IGNORE {
                continue;
            }
            specification.ignore.push(i as char);
        }
        if let Some((col, end)) = self.wrap() {
            specification.wrap.width = col;
            specification.wrap.separator = core::str::from_utf8(end).unwrap().to_owned();
        }
        for i in 0 .. 128u8 {
            let canonical = if self.val()[i as usize] < 1 << self.bit() {
                self.sym()[self.val()[i as usize] as usize]
            } else if self.val()[i as usize] == PADDING {
                self.pad().unwrap()
            } else {
                continue;
            };
            if i == canonical {
                continue;
            }
            specification.translate.from.push(i as char);
            specification.translate.to.push(canonical as char);
        }
        specification
    }

    #[doc(hidden)]
    #[must_use]
    pub const unsafe fn internal_new(implementation: &'static [u8]) -> DynEncoding {
        #[cfg(feature = "alloc")]
        let encoding = DynEncoding(Cow::Borrowed(implementation));
        #[cfg(not(feature = "alloc"))]
        let encoding = DynEncoding(implementation);
        encoding
    }

    #[doc(hidden)]
    #[must_use]
    pub fn internal_implementation(&self) -> &[u8] {
        &self.0
    }
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
        Ok(unsafe { cast(base) })
    }
}

impl<'a, Bit: BitWidth, Msb: Bool, Pad: Bool, Wrap: Bool, Ignore: Bool>
    From<&'a Encoding<Bit, Msb, Pad, Wrap, Ignore>> for &'a DynEncoding
{
    fn from(base: &'a Encoding<Bit, Msb, Pad, Wrap, Ignore>) -> Self {
        unsafe { &*core::ptr::from_ref(base).cast::<DynEncoding>() }
    }
}

/// Order in which bits are read from a byte
///
/// The base-conversion encoding is always little-endian. This means that the least significant
/// **byte** is always first. However, we can still choose whether, within a byte, this is the most
/// significant or the least significant **bit** that is first. If the terminology is confusing,
/// testing on an asymmetrical example should be enough to choose the correct value.
///
/// # Examples
///
/// In the following example, we can see that a base with the `MostSignificantFirst` bit-order has
/// the most significant bit first in the encoded output. In particular, the output is in the same
/// order as the bits in the byte. The opposite happens with the `LeastSignificantFirst` bit-order.
/// The least significant bit is first and the output is in the reverse order.
///
/// ```rust
/// use data_encoding_v3::{BitOrder, Specification};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("01");
/// spec.bit_order = BitOrder::MostSignificantFirst;  // default
/// let msb = spec.encoding().unwrap();
/// spec.bit_order = BitOrder::LeastSignificantFirst;
/// let lsb = spec.encoding().unwrap();
/// assert_eq!(msb.encode(&[0b01010011]), "01010011");
/// assert_eq!(lsb.encode(&[0b01010011]), "11001010");
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub enum BitOrder {
    /// Most significant bit first
    ///
    /// This is the most common and most intuitive bit-order. In particular, this is the bit-order
    /// used by [RFC4648] and thus the usual hexadecimal, base64, base32, base64url, and base32hex
    /// encodings. This is the default bit-order when [specifying](struct.Specification.html) a
    /// base.
    ///
    /// [RFC4648]: https://tools.ietf.org/html/rfc4648
    MostSignificantFirst,

    /// Least significant bit first
    ///
    /// # Examples
    ///
    /// DNSCurve [base32] uses least significant bit first:
    ///
    /// ```rust
    /// use data_encoding_v3::BASE32_DNSCURVE;
    /// assert_eq!(BASE32_DNSCURVE.encode(&[0x64, 0x88]), "4321");
    /// assert_eq!(BASE32_DNSCURVE.decode(b"4321").unwrap(), vec![0x64, 0x88]);
    /// ```
    ///
    /// [base32]: constant.BASE32_DNSCURVE.html
    LeastSignificantFirst,
}
#[cfg(feature = "alloc")]
use crate::BitOrder::*;

/// How to translate characters when decoding
///
/// The order matters. The first character of the `from` field is translated to the first character
/// of the `to` field. The second to the second. Etc.
///
/// See [Specification](struct.Specification.html) for more information.
#[derive(Debug, Clone)]
#[cfg(feature = "alloc")]
pub struct Translate {
    /// Characters to translate from
    pub from: String,

    /// Characters to translate to
    pub to: String,
}

/// How to wrap the output when encoding
///
/// See [Specification](struct.Specification.html) for more information.
#[derive(Debug, Clone)]
#[cfg(feature = "alloc")]
pub struct Wrap {
    /// Wrapping width
    ///
    /// Must be a multiple of:
    ///
    /// - 8 for a bit-width of 1 (binary), 3 (octal), and 5 (base32)
    /// - 4 for a bit-width of 2 (base4) and 6 (base64)
    /// - 2 for a bit-width of 4 (hexadecimal)
    ///
    /// Wrapping is disabled if null.
    pub width: usize,

    /// Wrapping characters
    ///
    /// Wrapping is disabled if empty.
    pub separator: String,
}

/// Base-conversion specification
///
/// It is possible to define custom encodings given a specification. To do so, it is important to
/// understand the theory first.
///
/// # Theory
///
/// Each subsection has an equivalent subsection in the [Practice](#practice) section.
///
/// ## Basics
///
/// The main idea of a [base-conversion] encoding is to see `[u8]` as numbers written in
/// little-endian base256 and convert them in another little-endian base. For performance reasons,
/// this crate restricts this other base to be of size 2 (binary), 4 (base4), 8 (octal), 16
/// (hexadecimal), 32 (base32), or 64 (base64). The converted number is written as `[u8]` although
/// it doesn't use all the 256 possible values of `u8`. This crate encodes to ASCII, so only values
/// smaller than 128 are allowed.
///
/// More precisely, we need the following elements:
///
/// - The bit-width N: 1 for binary, 2 for base4, 3 for octal, 4 for hexadecimal, 5 for base32, and
///   6 for base64
/// - The [bit-order](enum.BitOrder.html): most or least significant bit first
/// - The symbols function S from [0, 2<sup>N</sup>) (called values and written `uN`) to symbols
///   (represented as `u8` although only ASCII symbols are allowed, i.e. smaller than 128)
/// - The values partial function V from ASCII to [0, 2<sup>N</sup>), i.e. from `u8` to `uN`
/// - Whether trailing bits are checked: trailing bits are leading zeros in theory, but since
///   numbers are little-endian they come last
///
/// For the encoding to be correct (i.e. encoding then decoding gives back the initial input),
/// V(S(i)) must be defined and equal to i for all i in [0, 2<sup>N</sup>). For the encoding to be
/// [canonical][canonical] (i.e. different inputs decode to different outputs, or equivalently,
/// decoding then encoding gives back the initial input), trailing bits must be checked and if V(i)
/// is defined then S(V(i)) is equal to i for all i.
///
/// Encoding and decoding are given by the following pipeline:
///
/// ```text
/// [u8] <--1--> [[bit; 8]] <--2--> [[bit; N]] <--3--> [uN] <--4--> [u8]
/// 1: Map bit-order between each u8 and [bit; 8]
/// 2: Base conversion between base 2^8 and base 2^N (check trailing bits)
/// 3: Map bit-order between each [bit; N] and uN
/// 4: Map symbols/values between each uN and u8 (values must be defined)
/// ```
///
/// ## Extensions
///
/// All these extensions make the encoding not canonical.
///
/// ### Padding
///
/// Padding is useful if the following conditions are met:
///
/// - the bit-width is 3 (octal), 5 (base32), or 6 (base64)
/// - the length of the data to encode is not known in advance
/// - the data must be sent without buffering
///
/// Bases for which the bit-width N does not divide 8 may not concatenate encoded data. This comes
/// from the fact that it is not possible to make the difference between trailing bits and encoding
/// bits. Padding solves this issue by adding a new character to discriminate between trailing bits
/// and encoding bits. The idea is to work by blocks of lcm(8, N) bits, where lcm(8, N) is the least
/// common multiple of 8 and N. When such block is not complete, it is padded.
///
/// To preserve correctness, the padding character must not be a symbol.
///
/// ### Ignore characters when decoding
///
/// Ignoring characters when decoding is useful if after encoding some characters are added for
/// convenience or any other reason (like wrapping). In that case we want to first ignore those
/// characters before decoding.
///
/// To preserve correctness, ignored characters must not contain symbols or the padding character.
///
/// ### Wrap output when encoding
///
/// Wrapping output when encoding is useful if the output is meant to be printed in a document where
/// width is limited (typically 80-columns documents). In that case, the wrapping width and the
/// wrapping separator have to be defined.
///
/// To preserve correctness, the wrapping separator characters must be ignored (see previous
/// subsection). As such, wrapping separator characters must also not contain symbols or the padding
/// character.
///
/// ### Translate characters when decoding
///
/// Translating characters when decoding is useful when encoded data may be copied by a humain
/// instead of a machine. Humans tend to confuse some characters for others. In that case we want to
/// translate those characters before decoding.
///
/// To preserve correctness, the characters we translate _from_ must not contain symbols or the
/// padding character, and the characters we translate _to_ must only contain symbols or the padding
/// character.
///
/// # Practice
///
/// ## Basics
///
/// ```rust
/// use data_encoding_v3::{DynEncoding, Specification};
/// fn make_encoding(symbols: &str) -> DynEncoding {
///     let mut spec = Specification::new();
///     spec.symbols.push_str(symbols);
///     spec.encoding().unwrap()
/// }
/// let binary = make_encoding("01");
/// let octal = make_encoding("01234567");
/// let hexadecimal = make_encoding("0123456789abcdef");
/// assert_eq!(binary.encode(b"Bit"), "010000100110100101110100");
/// assert_eq!(octal.encode(b"Bit"), "20464564");
/// assert_eq!(hexadecimal.encode(b"Bit"), "426974");
/// ```
///
/// The `binary` base has 2 symbols `0` and `1` with value 0 and 1 respectively. The `octal` base
/// has 8 symbols `0` to `7` with value 0 to 7. The `hexadecimal` base has 16 symbols `0` to `9` and
/// `a` to `f` with value 0 to 15. The following diagram gives the idea of how encoding works in the
/// previous example (note that we can actually write such diagram only because the bit-order is
/// most significant first):
///
/// ```text
/// [      octal] |  2  :  0  :  4  :  6  :  4  :  5  :  6  :  4  |
/// [     binary] |0 1 0 0 0 0 1 0|0 1 1 0 1 0 0 1|0 1 1 1 0 1 0 0|
/// [hexadecimal] |   4   :   2   |   6   :   9   |   7   :   4   |
///                ^-- LSB                                       ^-- MSB
/// ```
///
/// Note that in theory, these little-endian numbers are read from right to left (the most
/// significant bit is at the right). Since leading zeros are meaningless (in our usual decimal
/// notation 0123 is the same as 123), it explains why trailing bits must be zero. Trailing bits may
/// occur when the bit-width of a base does not divide 8. Only binary, base4, and hexadecimal don't
/// have trailing bits issues. So let's consider octal and base64, which have trailing bits in
/// similar circumstances:
///
/// ```rust
/// use data_encoding_v3::{Specification, BASE64_NOPAD};
/// let octal = {
///     let mut spec = Specification::new();
///     spec.symbols.push_str("01234567");
///     spec.encoding().unwrap()
/// };
/// assert_eq!(BASE64_NOPAD.encode(b"B"), "Qg");
/// assert_eq!(octal.encode(b"B"), "204");
/// ```
///
/// We have the following diagram, where the base64 values are written between parentheses:
///
/// ```text
/// [base64] |   Q(16)   :   g(32)   : [has 4 zero trailing bits]
/// [ octal] |  2  :  0  :  4  :       [has 1 zero trailing bit ]
///          |0 1 0 0 0 0 1 0|0 0 0 0
/// [ ascii] |       B       |
///                           ^-^-^-^-- leading zeros / trailing bits
/// ```
///
/// ## Extensions
///
/// ### Padding
///
/// For octal and base64, lcm(8, 3) == lcm(8, 6) == 24 bits or 3 bytes. For base32, lcm(8, 5) is 40
/// bits or 5 bytes. Let's consider octal and base64:
///
/// ```rust
/// use data_encoding_v3::{Specification, BASE64};
/// let octal = {
///     let mut spec = Specification::new();
///     spec.symbols.push_str("01234567");
///     spec.padding = Some('=');
///     spec.encoding().unwrap()
/// };
/// // We start encoding but we only have "B" for now.
/// assert_eq!(BASE64.encode(b"B"), "Qg==");
/// assert_eq!(octal.encode(b"B"), "204=====");
/// // Now we have "it".
/// assert_eq!(BASE64.encode(b"it"), "aXQ=");
/// assert_eq!(octal.encode(b"it"), "322720==");
/// // By concatenating everything, we may decode the original data.
/// assert_eq!(BASE64.decode(b"Qg==aXQ=").unwrap(), b"Bit");
/// assert_eq!(octal.decode(b"204=====322720==").unwrap(), b"Bit");
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
/// ### Ignore characters when decoding
///
/// The typical use-case is to ignore newlines (`\r` and `\n`). But to keep the example small, we
/// will ignore spaces.
///
/// ```rust
/// let mut spec = data_encoding_v3::HEXLOWER.specification();
/// spec.ignore.push_str(" \t");
/// let base = spec.encoding().unwrap();
/// assert_eq!(base.decode(b"42 69 74"), base.decode(b"426974"));
/// ```
///
/// ### Wrap output when encoding
///
/// The typical use-case is to wrap after 64 or 76 characters with a newline (`\r\n` or `\n`). But
/// to keep the example small, we will wrap after 8 characters with a space.
///
/// ```rust
/// let mut spec = data_encoding_v3::BASE64.specification();
/// spec.wrap.width = 8;
/// spec.wrap.separator.push_str(" ");
/// let base64 = spec.encoding().unwrap();
/// assert_eq!(base64.encode(b"Hey you"), "SGV5IHlv dQ== ");
/// ```
///
/// Note that the output always ends with the separator.
///
/// ### Translate characters when decoding
///
/// The typical use-case is to translate lowercase to uppercase or reciprocally, but it is also used
/// for letters that look alike, like `O0` or `Il1`. Let's illustrate both examples.
///
/// ```rust
/// let mut spec = data_encoding_v3::HEXLOWER.specification();
/// spec.translate.from.push_str("ABCDEFOIl");
/// spec.translate.to.push_str("abcdef011");
/// let base = spec.encoding().unwrap();
/// assert_eq!(base.decode(b"BOIl"), base.decode(b"b011"));
/// ```
///
/// [base-conversion]: https://en.wikipedia.org/wiki/Positional_notation#Base_conversion
/// [canonical]: https://tools.ietf.org/html/rfc4648#section-3.5
#[derive(Debug, Clone)]
#[cfg(feature = "alloc")]
pub struct Specification {
    /// Symbols
    ///
    /// The number of symbols must be 2, 4, 8, 16, 32, or 64. Symbols must be ASCII characters
    /// (smaller than 128) and they must be unique.
    pub symbols: String,

    /// Bit-order
    ///
    /// The default is to use most significant bit first since it is the most common.
    pub bit_order: BitOrder,

    /// Check trailing bits
    ///
    /// The default is to check trailing bits. This field is ignored when unnecessary (i.e. for
    /// base2, base4, and base16).
    pub check_trailing_bits: bool,

    /// Padding
    ///
    /// The default is to not use padding. The padding character must be ASCII and must not be a
    /// symbol.
    pub padding: Option<char>,

    /// Characters to ignore when decoding
    ///
    /// The default is to not ignore characters when decoding. The characters to ignore must be
    /// ASCII and must not be symbols or the padding character.
    pub ignore: String,

    /// How to wrap the output when encoding
    ///
    /// The default is to not wrap the output when encoding. The wrapping characters must be ASCII
    /// and must not be symbols or the padding character.
    pub wrap: Wrap,

    /// How to translate characters when decoding
    ///
    /// The default is to not translate characters when decoding. The characters to translate from
    /// must be ASCII and must not have already been assigned a semantics. The characters to
    /// translate to must be ASCII and must have been assigned a semantics (symbol, padding
    /// character, or ignored character).
    pub translate: Translate,
}

#[cfg(feature = "alloc")]
impl Default for Specification {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg(feature = "alloc")]
enum SpecificationErrorImpl {
    BadSize,
    NotAscii,
    Duplicate(u8),
    ExtraPadding,
    WrapLength,
    WrapWidth(u8),
    FromTo,
    Undefined(u8),
}
#[cfg(feature = "alloc")]
use crate::SpecificationErrorImpl::*;

/// Specification error
#[derive(Debug, Copy, Clone)]
#[cfg(feature = "alloc")]
pub struct SpecificationError(SpecificationErrorImpl);

#[cfg(feature = "alloc")]
impl core::fmt::Display for SpecificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            BadSize => write!(f, "invalid number of symbols"),
            NotAscii => write!(f, "non-ascii character"),
            Duplicate(c) => write!(f, "{:?} has conflicting definitions", c as char),
            ExtraPadding => write!(f, "unnecessary padding"),
            WrapLength => write!(f, "invalid wrap width or separator length"),
            WrapWidth(x) => write!(f, "wrap width not a multiple of {x}"),
            FromTo => write!(f, "translate from/to length mismatch"),
            Undefined(c) => write!(f, "{:?} is undefined", c as char),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SpecificationError {
    fn description(&self) -> &str {
        match self.0 {
            BadSize => "invalid number of symbols",
            NotAscii => "non-ascii character",
            Duplicate(_) => "conflicting definitions",
            ExtraPadding => "unnecessary padding",
            WrapLength => "invalid wrap width or separator length",
            WrapWidth(_) => "wrap width not a multiple",
            FromTo => "translate from/to length mismatch",
            Undefined(_) => "undefined character",
        }
    }
}

#[cfg(feature = "alloc")]
impl Specification {
    /// Returns a default specification
    #[must_use]
    pub fn new() -> Specification {
        Specification {
            symbols: String::new(),
            bit_order: MostSignificantFirst,
            check_trailing_bits: true,
            padding: None,
            ignore: String::new(),
            wrap: Wrap { width: 0, separator: String::new() },
            translate: Translate { from: String::new(), to: String::new() },
        }
    }

    /// Returns the specified encoding
    ///
    /// # Errors
    ///
    /// Returns an error if the specification is invalid.
    pub fn encoding(&self) -> Result<DynEncoding, SpecificationError> {
        let symbols = self.symbols.as_bytes();
        let bit: u8 = match symbols.len() {
            2 => 1,
            4 => 2,
            8 => 3,
            16 => 4,
            32 => 5,
            64 => 6,
            _ => return Err(SpecificationError(BadSize)),
        };
        let mut values = [INVALID; 128];
        let set = |v: &mut [u8; 128], i: u8, x: u8| {
            check!(SpecificationError(NotAscii), i < 128);
            if v[i as usize] == x {
                return Ok(());
            }
            check!(SpecificationError(Duplicate(i)), v[i as usize] == INVALID);
            v[i as usize] = x;
            Ok(())
        };
        for (v, symbols) in symbols.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)] // no truncation
            set(&mut values, *symbols, v as u8)?;
        }
        let msb = self.bit_order == MostSignificantFirst;
        let ctb = self.check_trailing_bits || 8 % bit == 0;
        let pad = match self.padding {
            None => None,
            Some(pad) => {
                check!(SpecificationError(ExtraPadding), 8 % bit != 0);
                check!(SpecificationError(NotAscii), pad.len_utf8() == 1);
                set(&mut values, pad as u8, PADDING)?;
                Some(pad as u8)
            }
        };
        for i in self.ignore.bytes() {
            set(&mut values, i, IGNORE)?;
        }
        let wrap = if self.wrap.separator.is_empty() || self.wrap.width == 0 {
            None
        } else {
            let col = self.wrap.width;
            let end = self.wrap.separator.as_bytes();
            check!(SpecificationError(WrapLength), col < 256 && end.len() < 256);
            #[allow(clippy::cast_possible_truncation)] // no truncation
            let col = col as u8;
            #[allow(clippy::cast_possible_truncation)] // no truncation
            let dec = dec(bit as usize) as u8;
            check!(SpecificationError(WrapWidth(dec)), col % dec == 0);
            for &i in end {
                set(&mut values, i, IGNORE)?;
            }
            Some((col, end))
        };
        let from = self.translate.from.as_bytes();
        let to = self.translate.to.as_bytes();
        check!(SpecificationError(FromTo), from.len() == to.len());
        for i in 0 .. from.len() {
            check!(SpecificationError(NotAscii), to[i] < 128);
            let v = values[to[i] as usize];
            check!(SpecificationError(Undefined(to[i])), v != INVALID);
            set(&mut values, from[i], v)?;
        }
        let mut encoding = Vec::new();
        for _ in 0 .. 256 / symbols.len() {
            encoding.extend_from_slice(symbols);
        }
        encoding.extend_from_slice(&values);
        encoding.extend_from_slice(&[INVALID; 128]);
        match pad {
            None => encoding.push(INVALID),
            Some(pad) => encoding.push(pad),
        }
        encoding.push(bit);
        if msb {
            encoding[513] |= 0x08;
        }
        if ctb {
            encoding[513] |= 0x10;
        }
        if let Some((col, end)) = wrap {
            encoding.push(col);
            encoding.extend_from_slice(end);
        } else if values.contains(&IGNORE) {
            encoding.push(0);
        }
        Ok(DynEncoding(Cow::Owned(encoding)))
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

/// Lowercase hexadecimal encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, HEXLOWER};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789abcdef");
/// assert_eq!(HEXLOWER.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// # Examples
///
/// ```rust
/// use data_encoding_v3::HEXLOWER;
/// let deadbeef = vec![0xde, 0xad, 0xbe, 0xef];
/// assert_eq!(HEXLOWER.decode(b"deadbeef").unwrap(), deadbeef);
/// assert_eq!(HEXLOWER.encode(&deadbeef), "deadbeef");
/// ```
pub static HEXLOWER: Hex = unsafe { Hex::new_unchecked(HEXLOWER_IMPL) };
const HEXLOWER_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54,
    55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100,
    101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97,
    98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48,
    49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100,
    101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97,
    98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48,
    49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 97, 98, 99, 100, 101, 102, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 0, 1, 2,
    3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 10, 11, 12, 13, 14, 15, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 28,
];

/// Lowercase hexadecimal encoding with case-insensitive decoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, HEXLOWER_PERMISSIVE};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789abcdef");
/// spec.translate.from.push_str("ABCDEF");
/// spec.translate.to.push_str("abcdef");
/// assert_eq!(HEXLOWER_PERMISSIVE.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// # Examples
///
/// ```rust
/// use data_encoding_v3::HEXLOWER_PERMISSIVE;
/// let deadbeef = vec![0xde, 0xad, 0xbe, 0xef];
/// assert_eq!(HEXLOWER_PERMISSIVE.decode(b"DeadBeef").unwrap(), deadbeef);
/// assert_eq!(HEXLOWER_PERMISSIVE.encode(&deadbeef), "deadbeef");
/// ```
///
/// You can also define a shorter name:
///
/// ```rust
/// pub use data_encoding_v3::HEXLOWER_PERMISSIVE as HEX;
/// ```
pub static HEXLOWER_PERMISSIVE: Hex = unsafe { Hex::new_unchecked(HEXLOWER_PERMISSIVE_IMPL) };
const HEXLOWER_PERMISSIVE_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54,
    55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100,
    101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97,
    98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48,
    49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100,
    101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97,
    98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48,
    49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 97, 98, 99, 100, 101, 102, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 0, 1, 2,
    3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 128, 128, 128, 128, 10, 11, 12, 13, 14, 15, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 10, 11, 12, 13, 14, 15, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 28,
];

/// Uppercase hexadecimal encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, HEXUPPER};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEF");
/// assert_eq!(HEXUPPER.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It is compliant with [RFC4648] and known as "base16" or "hex".
///
/// # Examples
///
/// ```rust
/// use data_encoding_v3::HEXUPPER;
/// let deadbeef = vec![0xde, 0xad, 0xbe, 0xef];
/// assert_eq!(HEXUPPER.decode(b"DEADBEEF").unwrap(), deadbeef);
/// assert_eq!(HEXUPPER.encode(&deadbeef), "DEADBEEF");
/// ```
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-8
pub static HEXUPPER: Hex = unsafe { Hex::new_unchecked(HEXUPPER_IMPL) };
const HEXUPPER_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 128, 128, 128, 128, 10, 11,
    12, 13, 14, 15, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 28,
];

/// Uppercase hexadecimal encoding with case-insensitive decoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, HEXUPPER_PERMISSIVE};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEF");
/// spec.translate.from.push_str("abcdef");
/// spec.translate.to.push_str("ABCDEF");
/// assert_eq!(HEXUPPER_PERMISSIVE.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// # Examples
///
/// ```rust
/// use data_encoding_v3::HEXUPPER_PERMISSIVE;
/// let deadbeef = vec![0xde, 0xad, 0xbe, 0xef];
/// assert_eq!(HEXUPPER_PERMISSIVE.decode(b"DeadBeef").unwrap(), deadbeef);
/// assert_eq!(HEXUPPER_PERMISSIVE.encode(&deadbeef), "DEADBEEF");
/// ```
pub static HEXUPPER_PERMISSIVE: Hex = unsafe { Hex::new_unchecked(HEXUPPER_PERMISSIVE_IMPL) };
const HEXUPPER_PERMISSIVE_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 128, 128, 128, 128, 10, 11,
    12, 13, 14, 15, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 10, 11, 12, 13, 14, 15, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 28,
];

/// Padded base32 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE32};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");
/// spec.padding = Some('=');
/// assert_eq!(BASE32.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-6
pub static BASE32: Base32 = unsafe { Base32::new_unchecked(BASE32_IMPL) };
const BASE32_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72,
    73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55,
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72,
    73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55,
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 26, 27, 28, 29, 30, 31, 128, 128, 128, 128, 128, 130, 128, 128,
    128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 61, 29,
];

/// Unpadded base32 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE32_NOPAD};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");
/// assert_eq!(BASE32_NOPAD.as_dyn(), &spec.encoding().unwrap());
/// ```
pub static BASE32_NOPAD: Base32NoPad = unsafe { Base32NoPad::new_unchecked(BASE32_NOPAD_IMPL) };
const BASE32_NOPAD_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72,
    73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55,
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72,
    73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55,
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 50, 51, 52, 53, 54, 55, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 50, 51, 52, 53, 54, 55, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 26, 27, 28, 29, 30, 31, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 29,
];

/// Padded base32hex encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE32HEX};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEFGHIJKLMNOPQRSTUV");
/// spec.padding = Some('=');
/// assert_eq!(BASE32HEX.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-7
pub static BASE32HEX: Base32 = unsafe { Base32::new_unchecked(BASE32HEX_IMPL) };
const BASE32HEX_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 130, 128, 128, 128, 10, 11,
    12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 61, 29,
];

/// Unpadded base32hex encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE32HEX_NOPAD};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789ABCDEFGHIJKLMNOPQRSTUV");
/// assert_eq!(BASE32HEX_NOPAD.as_dyn(), &spec.encoding().unwrap());
/// ```
pub static BASE32HEX_NOPAD: Base32NoPad =
    unsafe { Base32NoPad::new_unchecked(BASE32HEX_NOPAD_IMPL) };
const BASE32HEX_NOPAD_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
    79, 80, 81, 82, 83, 84, 85, 86, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 128, 128, 128, 128, 10, 11,
    12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 29,
];

/// DNSSEC base32 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE32_DNSSEC};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789abcdefghijklmnopqrstuv");
/// spec.translate.from.push_str("ABCDEFGHIJKLMNOPQRSTUV");
/// spec.translate.to.push_str("abcdefghijklmnopqrstuv");
/// assert_eq!(BASE32_DNSSEC.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It conforms to [RFC5155]:
///
/// - It uses a base32 extended hex alphabet.
/// - It is case-insensitive when decoding and uses lowercase when encoding.
/// - It does not use padding.
///
/// [RFC5155]: https://tools.ietf.org/html/rfc5155
pub static BASE32_DNSSEC: Base32NoPad = unsafe { Base32NoPad::new_unchecked(BASE32_DNSSEC_IMPL) };
const BASE32_DNSSEC_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107,
    108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 103, 104,
    105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 48, 49, 50, 51, 52, 53,
    54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
    113, 114, 115, 116, 117, 118, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101,
    102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109,
    110, 111, 112, 113, 114, 115, 116, 117, 118, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106,
    107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 128, 128, 128, 128, 10, 11, 12, 13,
    14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 29,
];

#[allow(clippy::doc_markdown)]
/// DNSCurve base32 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{BitOrder, Specification, BASE32_DNSCURVE};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("0123456789bcdfghjklmnpqrstuvwxyz");
/// spec.bit_order = BitOrder::LeastSignificantFirst;
/// spec.translate.from.push_str("BCDFGHJKLMNPQRSTUVWXYZ");
/// spec.translate.to.push_str("bcdfghjklmnpqrstuvwxyz");
/// assert_eq!(BASE32_DNSCURVE.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It conforms to [DNSCurve].
///
/// [DNSCurve]: https://dnscurve.org/in-implement.html
pub static BASE32_DNSCURVE: Base32LsbNoPad =
    unsafe { Base32LsbNoPad::new_unchecked(BASE32_DNSCURVE_IMPL) };
const BASE32_DNSCURVE_IMPL: &[u8] = &[
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 98, 99, 100, 102, 103, 104, 106, 107, 108, 109, 110,
    112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
    98, 99, 100, 102, 103, 104, 106, 107, 108, 109, 110, 112, 113, 114, 115, 116, 117, 118, 119,
    120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 98, 99, 100, 102, 103, 104, 106, 107,
    108, 109, 110, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53,
    54, 55, 56, 57, 98, 99, 100, 102, 103, 104, 106, 107, 108, 109, 110, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 98, 99, 100, 102, 103,
    104, 106, 107, 108, 109, 110, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 98, 99, 100, 102, 103, 104, 106, 107, 108, 109, 110, 112, 113,
    114, 115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 98, 99,
    100, 102, 103, 104, 106, 107, 108, 109, 110, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121,
    122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 98, 99, 100, 102, 103, 104, 106, 107, 108, 109,
    110, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 128, 128, 128, 128, 128, 128, 128, 128, 10, 11,
    12, 128, 13, 14, 15, 128, 16, 17, 18, 19, 20, 128, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    128, 128, 128, 128, 128, 128, 128, 10, 11, 12, 128, 13, 14, 15, 128, 16, 17, 18, 19, 20, 128,
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 21,
];

/// Padded base64 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE64};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// spec.padding = Some('=');
/// assert_eq!(BASE64.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-4
pub static BASE64: Base64 = unsafe { Base64::new_unchecked(BASE64_IMPL) };
const BASE64_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114,
    115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67,
    68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67, 68,
    69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 62, 128, 128, 128, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 128, 128, 128, 130, 128,
    128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 128, 128, 128, 128, 128, 128, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 61, 30,
];

/// Unpadded base64 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE64_NOPAD};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// assert_eq!(BASE64_NOPAD.as_dyn(), &spec.encoding().unwrap());
/// ```
pub static BASE64_NOPAD: Base64NoPad = unsafe { Base64NoPad::new_unchecked(BASE64_NOPAD_IMPL) };
const BASE64_NOPAD_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114,
    115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67,
    68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67, 68,
    69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 62, 128, 128, 128, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 128, 128, 128, 128, 128,
    128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 128, 128, 128, 128, 128, 128, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 30,
];

/// MIME base64 encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE64_MIME};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// spec.padding = Some('=');
/// spec.wrap.width = 76;
/// spec.wrap.separator.push_str("\r\n");
/// assert_eq!(BASE64_MIME.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It does not exactly conform to [RFC2045] because it does not print the header
/// and does not ignore all characters.
///
/// [RFC2045]: https://tools.ietf.org/html/rfc2045
pub static BASE64_MIME: Base64Wrap = unsafe { Base64Wrap::new_unchecked(BASE64_MIME_IMPL) };
const BASE64_MIME_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114,
    115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67,
    68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67, 68,
    69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 129, 128, 128, 129, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 62, 128, 128, 128, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 128, 128, 128, 130, 128,
    128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 128, 128, 128, 128, 128, 128, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 61, 30, 76, 13, 10,
];

/// MIME base64 encoding without trailing bits check
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE64_MIME_PERMISSIVE};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
/// spec.padding = Some('=');
/// spec.wrap.width = 76;
/// spec.wrap.separator.push_str("\r\n");
/// spec.check_trailing_bits = false;
/// assert_eq!(BASE64_MIME_PERMISSIVE.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It does not exactly conform to [RFC2045] because it does not print the header
/// and does not ignore all characters.
///
/// [RFC2045]: https://tools.ietf.org/html/rfc2045
pub static BASE64_MIME_PERMISSIVE: Base64Wrap =
    unsafe { Base64Wrap::new_unchecked(BASE64_MIME_PERMISSIVE_IMPL) };
const BASE64_MIME_PERMISSIVE_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114,
    115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67,
    68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 65, 66, 67, 68,
    69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 43, 47, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 129, 128, 128, 129, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 62, 128, 128, 128, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 128, 128, 128, 130, 128,
    128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 128, 128, 128, 128, 128, 128, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 61, 14, 76, 13, 10,
];

/// Padded base64url encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE64URL};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_");
/// spec.padding = Some('=');
/// assert_eq!(BASE64URL.as_dyn(), &spec.encoding().unwrap());
/// ```
///
/// It conforms to [RFC4648].
///
/// [RFC4648]: https://tools.ietf.org/html/rfc4648#section-5
pub static BASE64URL: Base64 = unsafe { Base64::new_unchecked(BASE64URL_IMPL) };
const BASE64URL_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114,
    115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 65, 66, 67,
    68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 65, 66, 67, 68,
    69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 62, 128, 128, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 128, 128, 128, 130, 128,
    128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 128, 128, 128, 128, 63, 128, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 61, 30,
];

/// Unpadded base64url encoding
///
/// This encoding is a static version of:
///
/// ```rust
/// # use data_encoding_v3::{Specification, BASE64URL_NOPAD};
/// let mut spec = Specification::new();
/// spec.symbols.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_");
/// assert_eq!(BASE64URL_NOPAD.as_dyn(), &spec.encoding().unwrap());
/// ```
pub static BASE64URL_NOPAD: Base64NoPad =
    unsafe { Base64NoPad::new_unchecked(BASE64URL_NOPAD_IMPL) };
const BASE64URL_NOPAD_IMPL: &[u8] = &[
    65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
    89, 90, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114,
    115, 116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 65, 66, 67,
    68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97,
    98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    117, 118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 65, 66, 67, 68,
    69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 95, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 62, 128, 128, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 128, 128, 128, 128, 128,
    128, 128, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 128, 128, 128, 128, 63, 128, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 30,
];
