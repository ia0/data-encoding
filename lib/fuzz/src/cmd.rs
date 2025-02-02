#[cfg(not(fuzzing))]
use std::collections::HashMap;
#[cfg(not(fuzzing))]
use std::path::{Path, PathBuf};
#[cfg(not(fuzzing))]
use std::sync::OnceLock;

use data_encoding::{BitOrder, Encoding, Specification};

use crate::{gen, spec};

macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(not(fuzzing))]
        if *DEBUG.get().unwrap() {
            println!($($arg)*);
        }
    };
}

#[cfg(not(fuzzing))]
pub fn path(debug: bool) -> PathBuf {
    DEBUG.set(debug).unwrap();
    PathBuf::from(std::env::args_os().nth(1).unwrap())
}

#[cfg(not(fuzzing))]
pub fn target(path: &Path) -> String {
    path.components().nth(2).unwrap().as_os_str().to_str().unwrap().to_owned()
}

pub fn execute(target: &str, mut input: &[u8]) -> Output {
    let mut output = BothOutput::default();
    match target {
        "fuzz_any_spec" => {
            let Some(spec) = gen::any_spec(&mut input) else { return output.reject() };
            let Ok(base) = spec.encoding() else { return output.reject() };
            let spec = base.specification();
            stat_spec(&mut output, &spec, &base);
            let input = gen::rev_spec(&spec);
            assert_eq!(gen::spec(&mut input.as_slice()).encoding().unwrap(), base);
        }
        "impl_encode" => {
            let (spec, base) = gen_spec_base(&mut input, &mut output);
            assert_eq!(base.encode(input), spec::encode(&spec, input));
        }
        "impl_decode" => {
            let (spec, base) = gen_spec_base(&mut input, &mut output);
            let actual = base.decode(input);
            output.insert("decode_ok", actual.is_ok() as usize);
            assert_eq!(actual.ok(), spec::decode(&spec, input));
        }
        "impl_encode_write_buffer" => {
            let (_, base) = gen_spec_base(&mut input, &mut output);
            let mut buffer = vec![0; gen::nat(&mut input, 510, 2050)];
            output.insert("buffer_len", buffer.len());
            let mut actual = String::new();
            base.encode_write_buffer(input, &mut actual, &mut buffer).unwrap();
            assert_eq!(actual, base.encode(input));
        }
        "impl_new_encoder" => {
            let (_, base) = gen_spec_base(&mut input, &mut output);
            let mut actual = String::new();
            let mut full = Vec::new();
            let mut encoder = base.new_encoder(&mut actual);
            let mut num_chunks = 0;
            while !input.is_empty() {
                let len = gen::nat(&mut input, 0, 3 * 256 - 1);
                let chunk = gen::bytes(&mut input, len);
                full.extend_from_slice(chunk);
                encoder.append(chunk);
                num_chunks += 1;
            }
            encoder.finalize();
            output.insert("full_len", full.len());
            output.insert("num_chunks", num_chunks);
            assert_eq!(actual, base.encode(&full));
        }
        "spec_decode_encode" => {
            let (_, base) = gen_spec_base(&mut input, &mut output);
            let true = base.is_canonical() else { return output.reject() };
            let Ok(tmp) = base.decode(input) else { return output.reject() };
            assert_eq!(base.encode(&tmp).as_bytes(), input);
        }
        "spec_encode_decode" => {
            let (_, base) = gen_spec_base(&mut input, &mut output);
            assert_eq!(base.decode(base.encode(input).as_bytes()).unwrap(), input);
        }
        "spec_spec_base" => {
            let (_, base) = gen_spec_base(&mut input, &mut output);
            assert_eq!(base.specification().encoding().unwrap(), base);
        }
        x => unimplemented!("{x:?}"),
    }
    output.0
}

fn gen_spec_base(input: &mut &[u8], output: &mut BothOutput) -> (Specification, Encoding) {
    let base = gen::base(input);
    let spec = base.specification();
    debug!("{spec:#?}");
    debug!("{input:?}");
    stat_spec(output, &spec, &base);
    output.insert("input_len", input.len());
    (spec, base)
}

fn stat_spec(output: &mut BothOutput, spec: &Specification, base: &Encoding) {
    output.insert("bit", spec.symbols.len().trailing_zeros() as usize);
    output.insert("msb", (spec.bit_order == BitOrder::MostSignificantFirst) as usize);
    output.insert("ctb", spec.check_trailing_bits as usize);
    output.insert("pad", spec.padding.is_some() as usize);
    output.insert("ignore_len", spec.ignore.len());
    output.insert("wrap_col", spec.wrap.width);
    output.insert("wrap_len", spec.wrap.separator.len());
    output.insert("translate_len", spec.translate.from.len());
    output.insert("is_canonical", base.is_canonical() as usize);
}

#[cfg(fuzzing)]
type Output = libfuzzer_sys::Corpus;
#[cfg(not(fuzzing))]
type Output = HashMap<&'static str, usize>;

struct BothOutput(Output);

impl Default for BothOutput {
    fn default() -> Self {
        #[cfg(fuzzing)]
        let output = libfuzzer_sys::Corpus::Keep;
        #[cfg(not(fuzzing))]
        let output = HashMap::default();
        BothOutput(output)
    }
}

impl BothOutput {
    #[cfg(fuzzing)]
    fn insert(&mut self, _: &'static str, _: usize) {}
    #[cfg(not(fuzzing))]
    fn insert(&mut self, key: &'static str, value: usize) {
        assert!(self.0.insert(key, value).is_none());
    }

    #[cfg(fuzzing)]
    fn reject(self) -> Output {
        libfuzzer_sys::Corpus::Reject
    }
    #[cfg(not(fuzzing))]
    fn reject(self) -> Output {
        self.0
    }
}

#[cfg(not(fuzzing))]
static DEBUG: OnceLock<bool> = OnceLock::new();
