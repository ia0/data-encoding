//! Generic encoding module.

use base::{Base, mask, len, enc, dec};
use tool::{div_ceil, chunk, chunk_mut};

fn block<B: Base>(base: &B, input: &[u8], output: &mut [u8]) {
    let mut x = 0u64; // This is enough because `base.len() <= 40`.
    for j in 0 .. input.len() {
        x |= (input[j] as u64) << 8 * (enc(base) - 1 - j);
    }
    for j in 0 .. output.len() {
        let y = (x >> base.bit() * (dec(base) - 1 - j)) as u8;
        output[j] = base.sym(y & mask(base));
    }
}

fn last_block<B: Base>(base: &B, input: &[u8], output: &mut [u8]) {
    let ilen = input.len();
    let olen = div_ceil(8 * ilen, base.bit());
    let mut x = 0u64; // This is enough because `base.len() <= 40`.
    for j in 0 .. ilen {
        x |= (input[j] as u64) << 8 * (enc(base) - 1 - j);
    }
    for j in 0 .. olen {
        let y = (x >> base.bit() * (dec(base) - 1 - j)) as u8;
        output[j] = base.sym(y & mask(base));
    }
    if ilen != 0 {
        for j in olen .. dec(base) {
            output[j] = base.pad();
        }
    }
}

/// Converts an input length to its output length.
///
/// This function is meant to be used in conjunction with
/// [`encode_mut`](fn.encode_mut.html).
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn encode_len<B: Base>(base: &B, len: usize) -> usize {
    div_ceil(len, enc(base)) * dec(base)
}

/// Generic encoding function without allocation.
///
/// This function takes a base implementation, a shared input slice, a
/// mutable output slice, and encodes the input slice to the output
/// slice.
///
/// # Correctness
///
/// The base must satisfy the `Base` invariants.
///
/// # Panics
///
/// Panics if `output.len() != encode_len(input.len())`. May also
/// panic if `base` does not satisfy the `Base` invariants.
pub fn encode_mut<B: Base>(base: &B, input: &[u8], output: &mut [u8]) {
    let enc = enc(base);
    let dec = dec(base);
    let ilen = input.len();
    let olen = encode_len(base, ilen);
    assert_eq!(output.len(), olen);
    let n = ilen / enc;
    for i in 0 .. n {
        block(base, chunk(input, enc, i), chunk_mut(output, dec, i));
    }
    last_block(base, &input[enc * n ..], &mut output[dec * n ..]);
}

/// Generic encoding function with allocation.
///
/// This function is a wrapper for [`encode_mut`](fn.encode_mut.html)
/// that allocates an output of the correct size using
/// [`encode_len`](fn.encode_len.html).
///
/// # Correctness
///
/// The base must satisfy the `Base` invariants.
///
/// # Panics
///
/// May panic if `base` does not satisfy the `Base` invariants.
pub fn encode<B: Base>(base: &B, input: &[u8]) -> String {
    let mut output = vec![0u8; encode_len(base, input.len())];
    encode_mut(base, input, &mut output);
    unsafe {
        // This is valid because values are ascii.
        String::from_utf8_unchecked(output)
    }
}
