//! Generic decoding module.

use std::{error, fmt};

use base::{Base, len, enc, dec};
use tool::{div_ceil, chunk, chunk_mut, chunk_unchecked, chunk_mut_unchecked};

use self::Error::*;

fn decode_block<B: Base>
    (base: &B, input: &[u8], output: &mut [u8]) -> Result<(), Error>
{
    let mut x = 0u64; // This is enough because `base.len() <= 40`.
    for j in 0 .. input.len() {
        let y = try!(base.val(input[j]).ok_or(BadCharacter(j)));
        x |= (y as u64) << base.bit() * (dec(base) - 1 - j);
    }
    for j in 0 .. output.len() {
        output[j] = (x >> 8 * (enc(base) - 1 - j)) as u8;
    }
    Ok(())
}

fn decode_last<B: Base>
    (base: &B, input: &[u8], output: &mut [u8]) -> Result<usize, Error>
{
    let bit = base.bit();
    let enc = enc(base);
    let dec = dec(base);
    let mut r = 0;
    let mut x = 0u64; // This is enough because `base.len() <= 40`.
    for j in 0 .. dec {
        if bit * j / 8 > r {
            r += 1;
            if input[j] == base.pad() {
                for k in j .. dec {
                    check!(BadCharacter(k), input[k] == base.pad());
                }
                let s = bit * j - 8 * r;
                let p = (x >> 8 * (enc - 1 - r)) as u8 >> 8 - s;
                check!(BadPadding, p == 0);
                break;
            }
        }
        let y = try!(base.val(input[j]).ok_or(BadCharacter(j)));
        x |= (y as u64) << bit * (dec - 1 - j);
        if j == dec - 1 { r += 1; }
    }
    for j in 0 .. r {
        output[j] = (x >> 8 * (enc - 1 - j)) as u8;
    }
    Ok(r)
}

/// Converts an input length to its output length.
///
/// This function is meant to be used in conjunction with
/// [`decode_mut`](fn.decode_mut.html).
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn decode_len<B: Base>(base: &B, len: usize) -> usize {
    div_ceil(len, dec(base)) * enc(base)
}

/// Generic decoding function without allocation.
///
/// This function takes a base implementation, a shared input slice, a
/// mutable output slice, and decodes the input slice to the output
/// slice. It returns the length of the decoded data which may be
/// slightly smaller than the output length when input is padded.
///
/// # Correctness
///
/// The base must satisfy the `Base` invariants.
///
/// # Failures
///
/// Decoding may fail in the circumstances defined by
/// [`Error`](enum.Error.html).
///
/// # Panics
///
/// Panics if `output.len() != decode_len(input.len())`. May also
/// panic if `base` does not satisfy the `Base` invariants.
pub fn decode_mut<B: Base>
    (base: &B, input: &[u8], output: &mut [u8]) -> Result<usize, Error>
{
    let enc = enc(base);
    let dec = dec(base);
    let ilen = input.len();
    if ilen == 0 { return Ok(0); }
    if ilen % dec != 0 { return Err(BadLength); }
    assert_eq!(output.len(), decode_len(base, ilen));
    let n = ilen / dec - 1;
    for i in 0 .. n {
        let input = unsafe { chunk_unchecked(input, dec, i) };
        let output = unsafe { chunk_mut_unchecked(output, enc, i) };
        try!(decode_block(base, input, output)
             .map_err(|e| e.shift(dec * i)));
    }
    decode_last(base, chunk(input, dec, n), chunk_mut(output, enc, n))
        .map_err(|e| e.shift(dec * n))
        .map(|r| enc * n + r)
}

/// Generic decoding function with allocation.
///
/// This function is a wrapper for [`decode_mut`](fn.decode_mut.html)
/// that allocates an output of sufficient size using
/// [`decode_len`](fn.decode_len.html). The final size may be slightly
/// smaller if input is padded.
///
/// # Correctness
///
/// The base must satisfy the `Base` invariants.
///
/// # Failures
///
/// Decoding may fail in the circumstances defined by
/// [`Error`](enum.Error.html).
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn decode<B: Base>(base: &B, input: &[u8]) -> Result<Vec<u8>, Error> {
    let mut output = vec![0u8; decode_len(base, input.len())];
    let len = try!(decode_mut(base, input, &mut output));
    output.truncate(len);
    Ok(output)
}

/// Decoding errors.
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum Error {
    /// Bad input length.
    ///
    /// The input length is not a multiple of the decoding length,
    /// given by `dec(base)`.
    BadLength,

    /// Bad input character.
    ///
    /// The input does not contain only symbols and padding, or
    /// symbols and padding are at inappropriate positions. Only the
    /// last decoding block may contain padding and this padding must
    /// start at a valid position and be uninterrupted by symbols to
    /// the end of the block.
    BadCharacter(usize),

    /// Bad padding.
    ///
    /// The non-significant bits preceding padding and left out by
    /// decoding are non-zero.
    BadPadding,
}

impl Error {
    /// Increments error position.
    pub fn shift(self, delta: usize) -> Error {
        match self {
            BadCharacter(pos) => BadCharacter(pos + delta),
            other => other,
        }
    }

    /// Maps error position.
    pub fn map<F: FnOnce(usize) -> usize>(self, f: F) -> Error {
        match self {
            BadCharacter(pos) => BadCharacter(f(pos)),
            other => other,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &BadCharacter(p) => write!(f, "Unexpected character at offset {}", p),
            &BadLength => write!(f, "Unexpected length"),
            &BadPadding => write!(f, "Non-zero padding"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &BadCharacter(_) => "unexpected character",
            &BadLength => "unexpected length",
            &BadPadding => "non-zero padding",
        }
    }
}
