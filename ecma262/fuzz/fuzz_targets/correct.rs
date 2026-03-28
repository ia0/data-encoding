#![no_main]

use ecma262::{Alphabet, LastChunkHandling, spec};
use libfuzzer_sys::arbitrary::{self, Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;

struct Spec<'a> {
    alphabet: Alphabet,
    last_chunk_handling: LastChunkHandling,
    max_length: Option<usize>,
    input: &'a [u8],
}

impl<'a> std::fmt::Debug for Spec<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Spec")
            .field("alphabet", &self.alphabet)
            .field("last_chunk_handling", &self.last_chunk_handling)
            .field("max_length", &self.max_length)
            .field("input", &self.input.escape_ascii().to_string())
            .finish()
    }
}

impl<'a> Arbitrary<'a> for Spec<'a> {
    fn arbitrary(_: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        unreachable!()
    }

    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> arbitrary::Result<Self> {
        let alphabet = *u.choose(&[Alphabet::Base64, Alphabet::Base64Url])?;
        let last_chunk_handling = *u.choose(&[
            LastChunkHandling::Loose,
            LastChunkHandling::Strict,
            LastChunkHandling::StopBeforePartial,
        ])?;
        let max_length = if u.arbitrary::<bool>()? { Some(u.choose_index(1024)?) } else { None };
        let input = u.take_rest();
        Ok(Spec { alphabet, last_chunk_handling, max_length, input })
    }
}

fuzz_target!(|spec: Spec| {
    let Spec { alphabet, last_chunk_handling, max_length, input } = spec;
    let actual = ecma262::decode(input, alphabet, last_chunk_handling, max_length);
    let expected = spec::decode(input, alphabet, last_chunk_handling, max_length);
    assert_eq!(spec::Output::from(actual), expected);
});
