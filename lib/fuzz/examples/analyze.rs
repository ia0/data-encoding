#![feature(core_intrinsics)]

extern crate data_encoding;
extern crate data_encoding_fuzz;

use data_encoding::BitOrder;
use data_encoding::Specification;
use data_encoding_fuzz::generate_specification;
use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq)]
struct Config {
    bit: u8,
    msb: bool,
    ctb: bool,
    padding: bool,
    ignore: u8,
    wrap_width: u8,
    wrap_len: u8,
    translate: u8,
}

impl From<Specification> for Config {
    fn from(spec: Specification) -> Config {
        Config {
            bit: std::intrinsics::cttz(spec.symbols.len()) as u8,
            msb: spec.bit_order == BitOrder::MostSignificantFirst,
            ctb: spec.check_trailing_bits,
            padding: spec.padding.is_some(),
            ignore: spec.ignore.len() as u8,
            wrap_width: spec.wrap.width as u8,
            wrap_len: spec.wrap.separator.len() as u8,
            translate: spec.translate.from.len() as u8,
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "bit:{} msb:{} ctb:{} padding:{} ignore:{} wrap_width:{} wrap_len:{} translate:{}",
            self.bit,
            self.msb,
            self.ctb,
            self.padding,
            self.ignore,
            self.wrap_width,
            self.wrap_len,
            self.translate,
        )
    }
}

struct Display(Option<Config>);

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self.0 {
            None => write!(f, "none"),
            Some(config) => write!(f, "{}", config),
        }
    }
}

fn main() {
    let mut stats: HashMap<Option<Config>, usize> = HashMap::new();
    for entry in std::fs::read_dir(std::env::args().nth(1).unwrap()).unwrap() {
        let entry = entry.unwrap();
        let input = std::fs::read(entry.path()).unwrap();
        let config = generate_specification(&mut &input[..])
            .map(|spec| spec.encoding().unwrap().specification().into());
        *stats.entry(config).or_default() += 1;
    }
    for (config, count) in stats {
        println!("{} {}", count, Display(config));
    }
}
