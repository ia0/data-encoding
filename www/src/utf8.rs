use std::str::from_utf8;

fn to_hex(x: u8) -> char {
    (match x {
        0 ... 9 => b'0' + x,
        10 ... 15 => b'a' + x - 10,
        _ => panic!(),
    }) as char
}

fn from_hex(x: u8) -> u8 {
    match x {
        b'0' ... b'9' => x - b'0',
        b'A' ... b'F' => x - b'A' + 10,
        b'a' ... b'f' => x - b'a' + 10,
        _ => panic!(),
    }
}

fn escape_byte(input: u8, output: &mut String) {
    output.push('\\');
    output.push('x');
    output.push(to_hex(input >> 4));
    output.push(to_hex(input & 0xf));
}

fn escape_char(input: char, output: &mut String) {
    for c in input.escape_default() {
        output.push(c);
    }
}

pub fn encode(input: &[u8], whitespace: bool) -> String {
    let mut result = String::new();
    let mut pos = 0;
    let end = input.len();
    while pos < end {
        let (len, is_bad) = match from_utf8(&input[pos .. end]) {
            Ok(_) => (end - pos, false),
            Err(x) => (x.valid_up_to(), true),
        };
        for c in from_utf8(&input[pos .. pos + len]).unwrap().chars() {
            match c {
                '\n' => {
                    if whitespace {
                        escape_char(c, &mut result);
                    } else {
                        result.push(c);
                    }
                }
                '\t' | '\r' | '\\' => escape_char(c, &mut result),
                '\x00' ... '\x1f' | '\x7f' => escape_byte(c as u8, &mut result),
                _ => result.push(c),
            }
        }
        pos += len;
        if is_bad {
            escape_byte(input[pos], &mut result);
            pos += 1;
        }
    }
    result
}

pub fn decode(input: &str) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut input = input.bytes();
    let next = |input: &mut ::std::str::Bytes| {
        input.next().ok_or_else(|| format!("unexpected end of string"))
    };
    while let Some(x) = input.next() {
        if x != b'\\' {
            result.push(x);
            continue;
        }
        match next(&mut input)? {
            b't' => result.push(b'\t'),
            b'n' => result.push(b'\n'),
            b'r' => result.push(b'\r'),
            b'\\' => result.push(b'\\'),
            b'x' => {
                let high = from_hex(next(&mut input)?);
                let low = from_hex(next(&mut input)?);
                result.push(high << 4 | low);
            }
            x => return Err(format!("invalid escape sequence '\\{}'", x as char)),
        }
    }
    Ok(result)
}

#[test]
fn test_encode() {
    assert_eq!(encode(b"\x00", false), r"\x00");
    assert_eq!(encode(b"\\", false), r"\\");
    assert_eq!(encode(b"\x80", false), r"\x80");
    assert_eq!(encode(b"\xc3\xa9", false), "é");
    assert_eq!(encode(b"\xc3\xa9\x80\xc3\xa9A\x80A\x80", false), r"é\x80éA\x80A\x80");
    assert_eq!(encode(b"\n", false), "\n");
    assert_eq!(encode(b"\n", true), r"\n");
}

#[test]
fn test_decode() {
    assert_eq!(decode(r"\x00").unwrap(), b"\x00");
    assert_eq!(decode(r"\\").unwrap(), b"\\");
    assert_eq!(decode(r"\x80").unwrap(), b"\x80");
    assert_eq!(decode(r"é").unwrap(), b"\xc3\xa9");
    assert_eq!(decode(r"é\x80éA\x80A\x80").unwrap(), b"\xc3\xa9\x80\xc3\xa9A\x80A\x80");
    assert_eq!(decode("\n").unwrap(), b"\n");
    assert_eq!(decode("\x41").unwrap(), b"A");
    assert_eq!(decode("\x4A").unwrap(), b"J");
}
