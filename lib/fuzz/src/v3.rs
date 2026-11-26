//! Converts specifications from `data-encoding` to `data-encoding-v3`.

use data_encoding::{BitOrder, Specification};
use data_encoding_v3::{BitOrder as BitOrder3, Specification as Specification3};

pub fn spec(spec: &Specification) -> Specification3 {
    let mut spec3 = Specification3::new();
    spec3.symbols = spec.symbols.clone();
    spec3.bit_order = match spec.bit_order {
        BitOrder::MostSignificantFirst => BitOrder3::MostSignificantFirst,
        BitOrder::LeastSignificantFirst => BitOrder3::LeastSignificantFirst,
    };
    spec3.check_trailing_bits = spec.check_trailing_bits;
    spec3.padding = spec.padding;
    spec3.ignore = spec.ignore.clone();
    spec3.wrap.width = spec.wrap.width;
    spec3.wrap.separator = spec.wrap.separator.clone();
    spec3.translate.from = spec.translate.from.clone();
    spec3.translate.to = spec.translate.to.clone();
    spec3
}
