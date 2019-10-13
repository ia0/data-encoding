extern crate data_encoding;

use data_encoding::{DecodePartial, Encoding, Specification};

pub fn generate_encoding(data: &mut &[u8]) -> Encoding {
    generate_specification(data).encoding().unwrap()
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

    fn next_used(&self, data: &mut &[u8]) -> char {
        Ascii::next(&self.used, data)
    }

    fn next(input: &[u8], data: &mut &[u8]) -> char {
        input[generate(data, 0, input.len() as u8 - 1) as usize] as char
    }

    fn len_free(&self) -> u8 {
        self.free.len() as u8
    }
}

pub fn generate_specification(data: &mut &[u8]) -> Specification {
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
            _ => panic!(),
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

pub fn decode_prefix(encoding: &Encoding, input: &mut &[u8]) -> Vec<u8> {
    if let Err(e) = encoding.decode_len(input.len()) {
        *input = &input[.. e.position];
    }
    let mut output = vec![0; encoding.decode_len(input.len()).unwrap()];
    match encoding.decode_mut(input, &mut output) {
        Ok(len) => output.truncate(len),
        Err(DecodePartial { read, written, .. }) => {
            *input = &input[.. read];
            output.truncate(written)
        }
    }
    output
}
