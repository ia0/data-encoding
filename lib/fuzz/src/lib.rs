extern crate data_encoding;

use data_encoding::{Encoding, Specification};

pub fn generate_encoding(data: &mut &[u8]) -> Option<Encoding> {
    let spec = generate_specification(data)?;
    Some(spec.encoding().unwrap())
}

struct Ascii {
    free: Vec<u8>,
    used: Vec<u8>,
}

impl Ascii {
    fn new() -> Ascii {
        Ascii { free: (0 .. 128).collect(), used: Vec::with_capacity(128) }
    }

    fn next_free(&mut self, data: &mut &[u8]) -> Option<char> {
        let res = self.free.swap_remove(generate(data, 0, self.len_free() - 1)? as usize);
        self.used.push(res);
        Some(res as char)
    }

    fn next_used(&self, data: &mut &[u8]) -> Option<char> {
        Ascii::next(&self.used, data)
    }

    fn next(input: &[u8], data: &mut &[u8]) -> Option<char> {
        Some(input[generate(data, 0, input.len() as u8 - 1)? as usize] as char)
    }

    fn len_free(&self) -> u8 {
        self.free.len() as u8
    }
}

pub fn generate_specification(data: &mut &[u8]) -> Option<Specification> {
    let mut spec = Specification::new();
    let mut ascii = Ascii::new();
    let bit = generate(data, 1, 6)?;
    for _ in 0 .. 1 << bit {
        spec.symbols.push(ascii.next_free(data)?);
    }
    if generate(data, 0, 1)? == 1 {
        spec.bit_order = data_encoding::BitOrder::LeastSignificantFirst;
    }
    if generate(data, 0, 1)? == 1 {
        spec.check_trailing_bits = false;
    }
    if 8 % bit != 0 && generate(data, 0, 1)? == 1 {
        spec.padding = Some(ascii.next_free(data)?);
    }
    for _ in 0 .. generate(data, 0, ascii.len_free())? {
        spec.ignore.push(ascii.next_free(data)?);
    }
    if !spec.ignore.is_empty() {
        spec.wrap.width = generate(data, 0, 255)? as usize;
        if spec.wrap.width > 0 {
            let dec = match bit {
                1 | 3 | 5 => 8,
                2 | 6 => 4,
                4 => 2,
                _ => panic!(),
            };
            if spec.wrap.width % dec != 0 {
                return None;
            }
            for _ in 0 .. generate(data, 1, 255)? {
                spec.wrap.separator.push(Ascii::next(spec.ignore.as_bytes(), data)?);
            }
        }
    }
    for _ in 0 .. generate(data, 0, ascii.len_free())? {
        spec.translate.to.push(ascii.next_used(data)?);
    }
    for _ in 0 .. spec.translate.to.len() {
        spec.translate.from.push(ascii.next_free(data)?);
    }
    Some(spec)
}

fn generate(data: &mut &[u8], min: u8, max: u8) -> Option<u8> {
    if data.is_empty() {
        return None;
    }
    let mut res = data[0];
    if min > 0 || max < 255 {
        res = min + data[0] % (max - min + 1);
    }
    *data = &data[1 ..];
    Some(res)
}
