use data_encoding::{Encoding, Specification};

pub fn base(data: &mut &[u8]) -> Encoding {
    spec(data).encoding().unwrap()
}

pub fn spec(data: &mut &[u8]) -> Specification {
    let mut spec = Specification::new();
    let mut ascii = Ascii::new();
    let bit = generate(data, 1, 6);
    for _ in 0 .. 1 << bit {
        spec.symbols.push(ascii.next_free(data));
    }
    if generate(data, 0, 1) == 1 {
        spec.bit_order = data_encoding::BitOrder::LeastSignificantFirst;
    }
    if generate(data, 0, 1) == 1 {
        spec.check_trailing_bits = false;
    }
    if 8 % bit != 0 && generate(data, 0, 1) == 1 {
        spec.padding = Some(ascii.next_free(data));
    }
    let ignore_translate_len = generate(data, 0, ascii.len_free());
    let ignore_len = generate(data, 0, ignore_translate_len);
    let translate_len = ignore_translate_len - ignore_len;
    for _ in 0 .. ignore_len {
        spec.ignore.push(ascii.next_free(data));
    }
    if !spec.ignore.is_empty() {
        let dec = match bit {
            1 | 3 | 5 => 8,
            2 | 6 => 4,
            4 => 2,
            _ => unreachable!(),
        };
        spec.wrap.width = generate(data, 0, 255) as usize / dec * dec;
        if spec.wrap.width > 0 {
            for _ in 0 .. generate(data, 1, 255) {
                spec.wrap.separator.push(Ascii::next(spec.ignore.as_bytes(), data));
            }
        }
    }
    for _ in 0 .. translate_len {
        spec.translate.to.push(ascii.next_used(data));
    }
    for _ in 0 .. translate_len {
        spec.translate.from.push(ascii.next_free(data));
    }
    spec
}

pub fn rev_spec(spec: &Specification) -> Vec<u8> {
    assert!(spec.encoding().is_ok());
    let mut output = Vec::new();
    let mut ascii = Ascii::new();
    let bit = spec.symbols.len().trailing_zeros() as u8;
    output.push(bit - 1);
    for x in spec.symbols.bytes() {
        output.push(ascii.rev_free(x));
    }
    output.push((spec.bit_order == data_encoding::BitOrder::LeastSignificantFirst) as u8);
    output.push(!spec.check_trailing_bits as u8);
    if 8 % bit != 0 {
        output.push(spec.padding.is_some() as u8);
        if let Some(pad) = spec.padding {
            output.push(ascii.rev_free(pad as u8));
        }
    }
    output.push((spec.ignore.len() + spec.translate.from.len()) as u8);
    output.push(spec.ignore.len() as u8);
    for x in spec.ignore.bytes() {
        output.push(ascii.rev_free(x));
    }
    if !spec.ignore.is_empty() {
        output.push(spec.wrap.width as u8);
        if 0 < spec.wrap.width {
            output.push(spec.wrap.separator.len() as u8 - 1);
            for x in spec.wrap.separator.bytes() {
                output.push(Ascii::rev(spec.ignore.as_bytes(), x));
            }
        }
    }
    for x in spec.translate.to.bytes() {
        output.push(ascii.rev_used(x));
    }
    for x in spec.translate.from.bytes() {
        output.push(ascii.rev_free(x));
    }
    output
}

pub fn any_spec(data: &mut &[u8]) -> Option<Specification> {
    let symbols = string(data)?;
    let bit_order = match flip(data) {
        false => data_encoding::BitOrder::LeastSignificantFirst,
        true => data_encoding::BitOrder::MostSignificantFirst,
    };
    let check_trailing_bits = flip(data);
    let padding = string(data)?.pop();
    let ignore = string(data)?;
    let width = generate(data, 0, 255) as usize;
    let separator = string(data)?;
    let wrap = data_encoding::Wrap { width, separator };
    let from = string(data)?;
    let to = string(data)?;
    let translate = data_encoding::Translate { from, to };
    Some(Specification {
        symbols,
        bit_order,
        check_trailing_bits,
        padding,
        ignore,
        wrap,
        translate,
    })
}

pub fn bytes<'a>(data: &'_ mut &'a [u8], len: usize) -> &'a [u8] {
    let len = std::cmp::min(len, data.len());
    let res = &data[.. len];
    *data = &data[len ..];
    res
}

pub fn nat(data: &mut &[u8], min: usize, max: usize) -> usize {
    let log = match (max - min).checked_ilog2() {
        None => return min,
        Some(x) => x,
    };
    let mut res = 0;
    for _ in 0 .. log / 8 + 1 {
        res = (res << 8) | generate(data, 0, 255) as usize;
    }
    if usize::MIN < min || max < usize::MAX {
        res = min + res % (max - min + 1);
    }
    res
}

fn flip(data: &mut &[u8]) -> bool {
    generate(data, 0, 1) == 1
}

fn string(data: &mut &[u8]) -> Option<String> {
    let len = generate(data, 0, 255) as usize;
    String::from_utf8(bytes(data, len).to_vec()).ok()
}

fn generate(data: &mut &[u8], min: u8, max: u8) -> u8 {
    if data.is_empty() {
        return min;
    }
    let mut res = data[0];
    if min > 0 || max < 255 {
        res = min + data[0] % (max - min + 1);
    }
    *data = &data[1 ..];
    res
}

struct Ascii {
    free: Vec<u8>,
    used: Vec<u8>,
}

impl Ascii {
    fn new() -> Ascii {
        Ascii { free: (0 .. 128).collect(), used: Vec::with_capacity(128) }
    }

    fn next_free(&mut self, data: &mut &[u8]) -> char {
        let res = self.free.swap_remove(generate(data, 0, self.len_free() - 1) as usize);
        self.used.push(res);
        res as char
    }

    fn rev_free(&mut self, x: u8) -> u8 {
        let i = self.free.iter().position(|&y| x == y).unwrap();
        assert_eq!(self.free.swap_remove(i), x);
        self.used.push(x);
        i as u8
    }

    fn next_used(&self, data: &mut &[u8]) -> char {
        Ascii::next(&self.used, data)
    }

    fn rev_used(&self, x: u8) -> u8 {
        Ascii::rev(&self.used, x)
    }

    fn next(input: &[u8], data: &mut &[u8]) -> char {
        input[generate(data, 0, input.len() as u8 - 1) as usize] as char
    }

    fn rev(input: &[u8], x: u8) -> u8 {
        input.iter().position(|&y| x == y).unwrap() as u8
    }

    fn len_free(&self) -> u8 {
        self.free.len() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nat_ok() {
        #[track_caller]
        fn test(mut data: &[u8], min: usize, max: usize, expected: usize) {
            assert_eq!(nat(&mut data, min, max), expected);
            assert_eq!(data, &[]);
        }
        test(&[], 0, 0, 0);
        test(&[], 0, 0xffff, 0);
        test(&[0], 0, 0xffff, 0);
        test(&[0x23], 0, 0xffff, 0x2300);
        test(&[0x23, 0x58], 0, 0xffff, 0x2358);
        test(&[0x23, 0x58], 0x10000, 0x1ffff, 0x12358);
        test(&[0], 0, 1, 0);
        test(&[1], 0, 1, 1);
        test(&[2], 0, 1, 0);
        test(&[128], 0, 255, 128);
        test(&[1, 0], 0, 256, 256);
    }
}
