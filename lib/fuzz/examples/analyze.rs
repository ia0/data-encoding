use std::collections::HashMap;
use std::ops::AddAssign;

use data_encoding::BitOrder;
use data_encoding_fuzz::{decode_prefix, generate_specification};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Key {
    Bit,
    Msb,
    Ctb,
    Pad,
    HasIgnore,
    Ignore,
    HasWrap,
    WrapWidth,
    WrapLen,
    HasTranslate,
    Translate,
    Canonical,
    InputLen,
    DecodeLen,
}

const ALL_KEYS: &'static [Key] = {
    use Key::*;
    &[
        Bit,
        Msb,
        Ctb,
        Pad,
        HasIgnore,
        Ignore,
        HasWrap,
        WrapWidth,
        WrapLen,
        HasTranslate,
        Translate,
        Canonical,
        InputLen,
        DecodeLen,
    ]
};

impl Key {
    fn name(self) -> &'static str {
        use Key::*;
        match self {
            Bit => "bit",
            Msb => "msb",
            Ctb => "ctb",
            Pad => "pad",
            HasIgnore => "has_ignore",
            Ignore => "ignore",
            HasWrap => "has_wrap",
            WrapWidth => "wrap_width",
            WrapLen => "wrap_len",
            HasTranslate => "has_translate",
            Translate => "translate",
            Canonical => "canonical",
            InputLen => "input_len",
            DecodeLen => "decode_len",
        }
    }
}

#[derive(Default, Clone)]
struct Stat<T>(HashMap<Key, T>);

impl Stat<usize> {
    fn new(mut data: &[u8]) -> Stat<usize> {
        let encoding = generate_specification(&mut data).encoding().unwrap();
        let spec = encoding.specification();
        let mut stat = HashMap::new();
        assert!(stat.insert(Key::Bit, spec.symbols.len().trailing_zeros() as usize).is_none());
        assert!(stat
            .insert(Key::Msb, (spec.bit_order == BitOrder::MostSignificantFirst) as usize)
            .is_none());
        assert!(stat.insert(Key::Ctb, spec.check_trailing_bits as usize).is_none());
        assert!(stat.insert(Key::Pad, spec.padding.is_some() as usize).is_none());
        assert!(stat.insert(Key::HasIgnore, !spec.ignore.is_empty() as usize).is_none());
        assert!(stat.insert(Key::Ignore, spec.ignore.len()).is_none());
        assert!(stat.insert(Key::HasWrap, (spec.wrap.width > 0) as usize).is_none());
        assert!(stat.insert(Key::WrapWidth, spec.wrap.width).is_none());
        assert!(stat.insert(Key::WrapLen, spec.wrap.separator.len()).is_none());
        assert!(stat.insert(Key::HasTranslate, !spec.translate.from.is_empty() as usize).is_none());
        assert!(stat.insert(Key::Translate, spec.translate.from.len()).is_none());
        assert!(stat.insert(Key::Canonical, encoding.is_canonical() as usize).is_none());
        assert!(stat.insert(Key::InputLen, data.len()).is_none());
        decode_prefix(&encoding, &mut data);
        assert!(stat.insert(Key::DecodeLen, data.len()).is_none());
        Stat(stat)
    }
}

impl<T> Stat<T> {
    fn map<U>(&self, mut f: impl FnMut(&T) -> U) -> Stat<U> {
        Stat(self.0.iter().map(|(&k, x)| (k, f(x))).collect())
    }
}

impl<T: AddAssign + Default> AddAssign for Stat<T> {
    fn add_assign(&mut self, rhs: Stat<T>) {
        for (k, x) in rhs.0 {
            *self.0.entry(k).or_default() += x;
        }
    }
}

#[derive(Default, Clone)]
struct Stats {
    sum: Stat<f64>,
    count: usize,
}

impl Stats {
    fn add(&mut self, stat: &Stat<usize>) {
        self.sum += stat.map(|&x| x as f64);
        self.count += 1;
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "count: {}", self.count)?;
        for key in ALL_KEYS {
            let sum = *self.sum.0.get(key).unwrap();
            writeln!(f, "{}: {:.2}", key.name(), sum / self.count as f64)?;
        }
        Ok(())
    }
}

fn main() {
    let mut stats = vec![Stats::default(); 6];
    for entry in std::fs::read_dir(std::env::args().nth(1).unwrap()).unwrap() {
        let entry = entry.unwrap();
        let stat = Stat::new(&std::fs::read(entry.path()).unwrap());
        let bit = *stat.0.get(&Key::Bit).unwrap();
        stats[bit - 1].add(&stat);
    }
    for stats in &stats {
        println!("{}", stats);
    }
}
