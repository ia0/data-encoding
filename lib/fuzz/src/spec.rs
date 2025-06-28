//! Reference implementation of the specification.

use data_encoding::{BitOrder, Specification};

pub fn encode(spec: &Specification, input: &[u8]) -> String {
    // Make sure the specification is valid.
    assert!(spec.encoding().is_ok());
    // Define short variables.
    let symbols = spec.symbols.as_bytes();
    let bit = symbols.len().trailing_zeros() as usize;
    let msb = spec.bit_order == BitOrder::MostSignificantFirst;
    // Convert from base256 to binary and from binary to baseX.
    let mut output = bits_value(bit, msb, &value_bits(8, msb, input));
    // Convert from values to symbols.
    output.iter_mut().for_each(|x| *x = symbols[*x as usize]);
    // Pad to the next `dec(bit)` boundary, if needed.
    if let Some(pad) = spec.padding {
        while !output.len().is_multiple_of(dec(bit)) {
            output.push(pad as u8);
        }
    }
    // Wrap every `width` bytes with `separator`, if needed. Including a possibly partial last row.
    if spec.wrap.width != 0 {
        for row in std::mem::take(&mut output).chunks(spec.wrap.width) {
            output.extend_from_slice(row);
            output.extend_from_slice(spec.wrap.separator.as_bytes());
        }
    }
    // Cast the symbols to a string.
    String::from_utf8(output).unwrap()
}

pub fn decode(spec: &Specification, input: &[u8]) -> Option<Vec<u8>> {
    // Make sure the specification is valid.
    assert!(spec.encoding().is_ok());
    // Define short variables.
    let symbols = spec.symbols.as_bytes();
    let bit = symbols.len().trailing_zeros() as usize;
    let xlate = &spec.translate;
    // Make sure we also ignore the separators.
    let mut ignore = spec.ignore.as_bytes().to_vec();
    ignore.extend_from_slice(spec.wrap.separator.as_bytes());
    // Translate and ignore bytes as needed. Only symbols and padding are left (for valid input).
    let input: Vec<u8> = input
        .iter()
        .map(|&x| xlate.from.bytes().position(|y| y == x).map_or(x, |i| xlate.to.as_bytes()[i]))
        .filter(|x| !ignore.contains(x))
        .collect();
    // Decode by blocks of `dec(bit)` bytes. Only the last one may be partial.
    let mut output = Vec::new();
    for block in input.chunks(dec(bit)) {
        output.extend_from_slice(&decode_block(spec, block)?);
    }
    Some(output)
}

fn decode_block(spec: &Specification, mut input: &[u8]) -> Option<Vec<u8>> {
    // Define short variables.
    let bit = spec.symbols.len().trailing_zeros() as usize;
    let msb = spec.bit_order == BitOrder::MostSignificantFirst;
    // Remove padding, if needed.
    if let Some(pad) = spec.padding {
        // There are no partial blocks with padding.
        if input.len() != dec(bit) {
            return None;
        }
        // Repeatedly remove last byte, if padding.
        while *input.last()? == pad as u8 {
            input = &input[.. input.len() - 1];
        }
    }
    // Convert from symbols to values.
    let input = input.iter().map(|&x| value_symbol(spec, x)).collect::<Option<Vec<u8>>>()?;
    // Convert from baseX to binary.
    let mut bits = value_bits(bit, msb, &input);
    // Check trailing bits (leading bits of the binary number that don't form a full byte).
    let trail = bits.len() % 8;
    if 0 < trail {
        // The trailing bits should not contain a full symbol.
        if bit <= trail {
            return None;
        }
        // The trailing bits should be zero, if checked.
        let trail = bits.split_off(bits.len() - trail);
        if spec.check_trailing_bits && trail.iter().any(|x| *x) {
            return None;
        }
    }
    // A block cannot be composed of padding only.
    if bits.is_empty() {
        return None;
    }
    // Convert from binary to base256.
    Some(bits_value(8, msb, &bits))
}

fn value_symbol(spec: &Specification, symbol: u8) -> Option<u8> {
    // The value of a symbol is its position in the specification.
    spec.symbols.bytes().position(|x| x == symbol).map(|x| x as u8)
}

fn value_bits(bit: usize, msb: bool, input: &[u8]) -> Vec<bool> {
    // Convert from binary to baseX.
    let mut output = Vec::new();
    for &x in input {
        for i in order(msb, bit) {
            output.push(x & (1 << i) != 0);
        }
    }
    output
}

fn bits_value(bit: usize, msb: bool, input: &[bool]) -> Vec<u8> {
    // Convert from baseX to binary.
    let mut output = Vec::new();
    for bits in input.chunks(bit) {
        output.push(order(msb, bit).zip(bits).map(|(i, &b)| (b as u8) << i).sum());
    }
    output
}

fn order(msb: bool, n: usize) -> Box<dyn Iterator<Item = usize>> {
    // Iterate from 0 to n - 1, or the opposite if most significant bit first.
    if msb {
        Box::new((0 .. n).rev())
    } else {
        Box::new(0 .. n)
    }
}

fn enc(bit: usize) -> usize {
    // Input block size for encoding, output block size for decoding.
    match bit {
        1 | 2 | 4 => 1,
        3 | 6 => 3,
        5 => 5,
        _ => unreachable!(),
    }
}

fn dec(bit: usize) -> usize {
    // Input block size for decoding, output block size for encoding.
    enc(bit) * 8 / bit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_bits_ok() {
        #[track_caller]
        fn test(bit: usize, msb: bool, values: &[u8], bits: &[u8]) {
            let bits: Vec<_> = bits.into_iter().map(|&x| x == 1).collect();
            assert_eq!(value_bits(bit, msb, values), bits);
            assert_eq!(bits_value(bit, msb, &bits), values);
        }
        test(8, true, &[0xc5, 0x69], &[1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1]);
        test(8, false, &[0xc5, 0x69], &[1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 0]);
        test(6, true, &[0x36, 0x2c], &[1, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0]);
        test(6, false, &[0x36, 0x2c], &[0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1]);
    }
}
