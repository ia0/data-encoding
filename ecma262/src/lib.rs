//! Implements FromBase64 according to ECMA-262.

use data_encoding::{Character, DecodeError, DecodeKind, DecodePartial, Encoding};
use data_encoding_macro::new_encoding;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alphabet {
    Base64,
    Base64Url,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LastChunkHandling {
    Loose,
    Strict,
    StopBeforePartial,
}
use LastChunkHandling::*;

#[derive(Debug, PartialEq, Eq)]
pub struct DecodeMutResult {
    /// Number of bytes correctly read from the input.
    pub read: usize,

    /// Number of bytes correctly written to the output.
    pub written: usize,

    /// Whether an error was encountered.
    ///
    /// When an error occurs, the [`read`] and [`written`] fields are maximal with respect to
    /// complete blocks (except for white-spaces). In other words, if the first error is in block N
    /// (where a block does not end, but may start with, white-spaces), then all blocks less than N
    /// have been correctly decoded.
    pub error: Option<DecodeError>,
}

/// Decodes `input` in `output` according to the given parameters.
///
/// # Panics
///
/// Panics if `output.len() < 6 * input.len() / 8`. It is not an error if `max_length` is smaller
/// than `output.len()`. This function will however not optimize those cases.
pub fn decode_mut(
    input: &[u8], output: &mut [u8], alphabet: Alphabet, last_chunk_handling: LastChunkHandling,
    max_length: Option<usize>,
) -> DecodeMutResult {
    // Select the appropriate encoding.
    let base = match alphabet {
        Alphabet::Base64 => &BASE64,
        Alphabet::Base64Url => &BASE64URL,
    };
    let max_length = max_length.unwrap_or(usize::MAX);

    // Decode as much as possible.
    let (mut read, mut written) = match base.decode_mut(&input, &mut output[.. 6 * input.len() / 8])
    {
        Ok(olen) => (input.len(), olen),
        Err(DecodePartial { read, written, .. }) => (read, written),
    };

    // Backtrack to the last complete chunk that fits below the maximum output length.
    let extra_output = written - core::cmp::min(written, max_length) / 3 * 3;
    let mut extra_input = (8 * extra_output).div_ceil(6);
    written -= extra_output;
    loop {
        // Backtrack white-spaces.
        while 0 < read && base.interpret_byte(input[read - 1]).is_ignored() {
            read -= 1;
        }
        if extra_input == 0 {
            break;
        }
        // Backtrack one symbol.
        read -= 1;
        extra_input -= 1;
        debug_assert!(base.interpret_byte(input[read]).is_symbol());
    }

    // Parse the next chunk manually.
    let mut index = [0; 4]; // maps to index in input
    let mut index_len = 0;
    let mut index_pad = 4;
    let mut ipos = read;
    let remaining = max_length - written;
    if remaining == 0 {
        return DecodeMutResult { read, written, error: None };
    }
    while ipos < input.len() {
        let byte = input[ipos];
        let position = ipos;
        ipos += 1;
        let kind = match base.interpret_byte(byte) {
            Character::Padding => unreachable!(),
            Character::Ignored => continue,
            Character::Symbol { .. } if index_pad < 4 => Some(DecodeKind::Padding),
            Character::Symbol { .. } => None,
            Character::Invalid if byte != b'=' => Some(DecodeKind::Symbol),
            Character::Invalid if index_len < 2 => Some(DecodeKind::Padding),
            Character::Invalid => {
                index_pad = core::cmp::min(index_pad, index_len);
                None
            }
        };
        if let Some(kind) = kind {
            return DecodeMutResult { read, written, error: Some(DecodeError { position, kind }) };
        }
        if index_len == 4 {
            debug_assert!(index_pad < 4);
            let error = Some(DecodeError { position, kind: DecodeKind::Padding });
            return DecodeMutResult { read, written, error };
        }
        index[index_len] = position;
        index_len += 1;
        if matches!((core::cmp::min(index_len, index_pad), remaining), (3, 1) | (4, 2)) {
            return DecodeMutResult { read, written, error: None };
        }
    }
    debug_assert!(index_len <= 4 && index_pad <= 4);
    debug_assert!(index_len < 4 || index_pad < 4);

    // Process the last chunk.
    if index_len == 0 {
        return DecodeMutResult { read: input.len(), written, error: None };
    }
    let check = match (last_chunk_handling, index_len, index_pad) {
        (Loose, 1, _) | (Loose, 0 .. 4, 0 .. 4) | (Strict, 0 .. 4, _) => {
            let error = Some(DecodeError { position: ipos, kind: DecodeKind::Length });
            return DecodeMutResult { read, written, error };
        }
        (Loose, _, _) => false,
        (Strict, _, _) => true,
        (StopBeforePartial, 0 .. 4, _) => return DecodeMutResult { read, written, error: None },
        (StopBeforePartial, _, _) => false,
    };
    let iend = core::cmp::min(index_len, index_pad);
    let oend = iend - 1;
    let mut ichunk = [b'A'; 4];
    for i in 0 .. iend {
        ichunk[i] = input[index[i]];
    }
    let mut ochunk = [0; 3];
    let rchunk = base.decode_mut(&ichunk, &mut ochunk);
    debug_assert_eq!(rchunk, Ok(3));
    if check && iend < 4 && ochunk[oend] != 0 {
        let error = Some(DecodeError { position: index[iend], kind: DecodeKind::Trailing });
        return DecodeMutResult { read, written, error };
    }
    output[written ..][.. oend].copy_from_slice(&ochunk[.. oend]);
    read = input.len();
    written += oend;
    DecodeMutResult { read, written, error: None }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodeResult {
    /// Number of bytes correctly read from the input.
    pub read: usize,

    /// Decoded output corresponding to the read input.
    pub output: Vec<u8>,

    /// Whether an error was encountered.
    ///
    /// When an error occurs, the [`read`] and [`output`] fields are maximal with respect to
    /// complete blocks (except for white-spaces). In other words, if the first error is in block N
    /// (where a block does not end, but may start with, white-spaces), then all blocks less than N
    /// have been correctly decoded.
    pub error: Option<DecodeError>,
}

/// Decodes `input` in `output` according to the given parameters.
pub fn decode(
    input: &[u8], alphabet: Alphabet, last_chunk_handling: LastChunkHandling,
    max_length: Option<usize>,
) -> DecodeResult {
    let mut output = vec![0; 6 * input.len() / 8];
    let DecodeMutResult { read, written, error } =
        decode_mut(input, &mut output, alphabet, last_chunk_handling, max_length);
    debug_assert!(written <= output.len());
    output.truncate(written);
    DecodeResult { read, output, error }
}

/// Reference implementation.
pub mod spec {
    use data_encoding::BASE64_NOPAD;

    use super::LastChunkHandling::*;
    use super::{Alphabet, LastChunkHandling};

    #[derive(Debug, PartialEq, Eq)]
    pub struct Output {
        pub read: usize,
        pub bytes: Vec<u8>,
        pub error: bool,
    }

    impl From<super::DecodeResult> for Output {
        fn from(value: super::DecodeResult) -> Self {
            Output { read: value.read, bytes: value.output, error: value.error.is_some() }
        }
    }

    pub fn decode(
        input: &[u8], alphabet: Alphabet, last_chunk_handling: LastChunkHandling,
        max_length: Option<usize>,
    ) -> Output {
        let max_length = max_length.unwrap_or(usize::MAX);
        if max_length == 0 {
            return Output { read: 0, bytes: vec![], error: false };
        }
        let mut read = 0;
        let mut bytes = Vec::new();
        let mut chunk = Vec::new();
        let mut index = 0;
        loop {
            skip_whitespace(input, &mut index);
            if index == input.len() {
                if !chunk.is_empty() {
                    match last_chunk_handling {
                        StopBeforePartial => return Output { read, bytes, error: false },
                        Strict => return Output { read, bytes, error: true },
                        Loose => (),
                    }
                    if chunk.len() == 1 {
                        return Output { read, bytes, error: true };
                    }
                    bytes.extend_from_slice(&decode_final(&chunk, false).unwrap());
                }
                return Output { read: input.len(), bytes, error: false };
            }
            let mut char_ = input[index];
            index += 1;
            if char_ == b'=' {
                if chunk.len() < 2 {
                    return Output { read, bytes, error: true };
                }
                skip_whitespace(input, &mut index);
                if chunk.len() == 2 {
                    if index == input.len() {
                        if last_chunk_handling == StopBeforePartial {
                            return Output { read, bytes, error: false };
                        }
                        return Output { read, bytes, error: true };
                    }
                    char_ = input[index];
                    if char_ == b'=' {
                        index += 1;
                        skip_whitespace(input, &mut index);
                    }
                }
                if index < input.len() {
                    return Output { read, bytes, error: true };
                }
                let check = last_chunk_handling == Strict;
                let Some(result) = decode_final(&chunk, check) else {
                    return Output { read, bytes, error: true };
                };
                bytes.extend_from_slice(&result);
                return Output { read: input.len(), bytes, error: false };
            }
            if alphabet == Alphabet::Base64Url {
                match char_ {
                    b'+' | b'/' => return Output { read, bytes, error: true },
                    b'-' => char_ = b'+',
                    b'_' => char_ = b'/',
                    _ => (),
                }
            }
            if !matches!(char_, b'A' ..= b'Z' | b'a' ..= b'z' | b'0' ..= b'9' | b'+' | b'/') {
                return Output { read, bytes, error: true };
            }
            let remaining = max_length - bytes.len();
            if (remaining == 1 && chunk.len() == 2) || (remaining == 2 && chunk.len() == 3) {
                return Output { read, bytes, error: false };
            }
            chunk.push(char_);
            if chunk.len() == 4 {
                bytes.extend_from_slice(&decode_full(&chunk));
                chunk.clear();
                read = index;
                if bytes.len() == max_length {
                    return Output { read, bytes, error: false };
                }
            }
        }
    }

    fn skip_whitespace(input: &[u8], index: &mut usize) {
        while matches!(input.get(*index), Some(0x09 | 0x0a | 0x0c | 0x0d | 0x20)) {
            *index += 1;
        }
    }

    fn decode_final(input: &[u8], check: bool) -> Option<Vec<u8>> {
        let mut chunk = input.to_vec();
        if input.len() == 2 {
            chunk.extend_from_slice(b"AA");
        } else {
            assert_eq!(input.len(), 3);
            chunk.push(b'A');
        }
        let output = decode_full(&chunk);
        let len = input.len() - 1;
        (!check || output[len] == 0).then(|| output[.. len].to_vec())
    }

    fn decode_full(input: &[u8]) -> [u8; 3] {
        let mut output = [0; 3];
        assert_eq!(BASE64_NOPAD.decode_mut(input, &mut output), Ok(3));
        output
    }
}

const BASE64: Encoding = new_encoding! {
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    ignore: " \t\n\x0C\r",
    check_trailing_bits: false,
};

const BASE64URL: Encoding = new_encoding! {
    symbols: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
    ignore: " \t\n\x0C\r",
    check_trailing_bits: false,
};
