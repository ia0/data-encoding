use data_encoding::{Encoding, Specification};

fn advance<'a>(x: &mut &'a [u8], n: usize) -> Option<&'a [u8]> {
    if x.len() < n {
        return None;
    }
    let r = &x[.. n];
    *x = &x[n ..];
    Some(r)
}

fn encode_slice(input: &[u8], output: &mut Vec<u8>) {
    assert!(input.len() < 256);
    output.push(input.len() as u8);
    output.extend_from_slice(input);
}

fn decode_slice<'a>(input: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = advance(input, 1)?[0] as usize;
    advance(input, len)
}

fn encode_range(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    for (start, end) in ::range::encode_range(input) {
        match end {
            None => output.push(start),
            Some(end) => {
                output.push(0x80 | start);
                output.push(end);
            }
        }
    }
    output
}

fn decode_range(mut input: &[u8]) -> Option<String> {
    let mut output = Vec::new();
    while input.len() > 0 {
        if input[0] & 0x80 == 0 {
            output.push(input[0]);
            input = &input[1 ..];
        } else {
            if input.len() < 2 {
                return None;
            }
            for x in input[0] & 0x7f ..= input[1] {
                output.push(x);
            }
            input = &input[2 ..];
        }
    }
    String::from_utf8(output).ok()
}

#[test]
fn test_range() {
    let test_decode = |dec: &[u8], enc: Option<&str>| {
        assert_eq!(decode_range(dec).as_ref().map(String::as_str), enc);
    };
    let test = |dec, enc| {
        test_decode(dec, enc);
        if let Some(enc) = enc {
            assert_eq!(encode_range(enc.as_bytes()).as_slice(), dec);
        }
    };
    test(b"", Some(""));
    test(b"A", Some("A"));
    test(b"AZ", Some("AZ"));
    test(&[0x80 | b'A', b'E'], Some("ABCDE"));
    test_decode(b"ABC", Some("ABC"));
    test_decode(&[0x80], None);
}

pub fn encode_encoding(encoding: &Encoding) -> Vec<u8> {
    let spec = encoding.specification();
    let mut result = Vec::new();
    result.push(0);
    if spec.bit_order == ::data_encoding::BitOrder::LeastSignificantFirst {
        result[0] |= 1 << 5;
    }
    if !spec.check_trailing_bits {
        result[0] |= 1 << 4;
    }
    if let Some(padding) = spec.padding {
        result[0] |= 1 << 3;
        result.push(padding as u8);
    }
    if !spec.ignore.is_empty() {
        result[0] |= 1 << 2;
        encode_slice(&encode_range(spec.ignore.as_bytes()), &mut result);
    }
    if spec.wrap.width != 0 {
        result[0] |= 1 << 1;
        result.push(spec.wrap.width as u8);
        encode_slice(spec.wrap.separator.as_bytes(), &mut result);
    }
    if !spec.translate.from.is_empty() {
        result[0] |= 1 << 0;
        encode_slice(&encode_range(spec.translate.from.as_bytes()), &mut result);
        encode_slice(&encode_range(spec.translate.to.as_bytes()), &mut result);
    }
    result.extend_from_slice(&encode_range(spec.symbols.as_bytes()));
    result
}

pub fn decode_specification(mut input: &[u8]) -> Option<Specification> {
    let mut spec = Specification::new();
    let desc = advance(&mut input, 1)?[0];
    if desc & 1 << 5 != 0 {
        spec.bit_order = ::data_encoding::BitOrder::LeastSignificantFirst;
    }
    if desc & 1 << 4 != 0 {
        spec.check_trailing_bits = false;
    }
    if desc & 1 << 3 != 0 {
        spec.padding = Some(advance(&mut input, 1)?[0] as char);
    }
    if desc & 1 << 2 != 0 {
        spec.ignore = decode_range(decode_slice(&mut input)?)?;
    }
    if desc & 1 << 1 != 0 {
        spec.wrap.width = advance(&mut input, 1)?[0] as usize;
        spec.wrap.separator = ::std::str::from_utf8(decode_slice(&mut input)?)
            .ok()?
            .to_string();
    }
    if desc & 1 << 0 != 0 {
        spec.translate.from = decode_range(decode_slice(&mut input)?)?;
        spec.translate.to = decode_range(decode_slice(&mut input)?)?;
    }
    spec.symbols = decode_range(input)?;
    Some(spec)
}

pub fn decode_encoding(input: &[u8]) -> Option<Encoding> {
    decode_specification(input).and_then(|spec| spec.encoding().ok())
}

#[test]
fn test_encoding() {
    let test = |enc: &[u8], dec: Option<&Encoding>| {
        assert_eq!(decode_encoding(enc).as_ref(), dec);
        if let Some(dec) = dec {
            assert_eq!(encode_encoding(dec).as_slice(), enc);
        }
    };
    test(&[], None);
    assert!(decode_specification(&[0x00]).is_some());
    test(&[0x00], None);
    let mut spec = ::data_encoding::BASE64_MIME.specification();
    spec.check_trailing_bits = false;
    test(
        &[
            0x1e,
            b'=',
            2,
            b'\n',
            b'\r',
            76,
            2,
            b'\r',
            b'\n',
            0x80 | b'A',
            b'Z',
            0x80 | b'a',
            b'z',
            0x80 | b'0',
            b'9',
            b'+',
            b'/',
        ],
        Some(&spec.encoding().unwrap()),
    );
    test(
        &[
            0x21,
            8,
            0x80 | b'B',
            b'D',
            0x80 | b'F',
            b'H',
            0x80 | b'J',
            b'N',
            0x80 | b'P',
            b'Z',
            8,
            0x80 | b'b',
            b'd',
            0x80 | b'f',
            b'h',
            0x80 | b'j',
            b'n',
            0x80 | b'p',
            b'z',
            0x80 | b'0',
            b'9',
            0x80 | b'b',
            b'd',
            0x80 | b'f',
            b'h',
            0x80 | b'j',
            b'n',
            0x80 | b'p',
            b'z',
        ],
        Some(&::data_encoding::BASE32_DNSCURVE),
    );
}
