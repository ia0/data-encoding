#![no_main]

use data_encoding_fuzz::cmd;
use libfuzzer_sys::{fuzz_target, Corpus};

fuzz_target!(|data: &[u8]| -> Corpus { cmd::execute(env!("CARGO_BIN_NAME"), data) });
