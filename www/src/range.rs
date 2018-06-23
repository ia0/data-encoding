pub fn encode_range(mut input: &[u8]) -> Vec<(u8, Option<u8>)> {
    let mut output = Vec::new();
    while input.len() > 0 {
        if input[0] == b'-' {
            output.push((b'-', None));
        } else {
            let start = input[0];
            while input.len() > 1 && input[1] == input[0] + 1 && input[1] != b'-' {
                input = &input[1 ..];
            }
            if input[0] == start {
                output.push((start, None));
            } else {
                output.push((start, Some(input[0])));
            }
        }
        input = &input[1 ..];
    }
    output
}

fn ensure_ascii(x: u8) -> Result<char, String> {
    if x < 128 {
        Ok(x as char)
    } else {
        Err("non-ASCII byte".to_string())
    }
}

pub fn encode(input: &str) -> Result<String, String> {
    let mut output = String::new();
    for (start, end) in encode_range(input.as_bytes()) {
        if start == b'-' {
            assert_eq!(end, None);
            output.push_str("--");
            continue;
        }
        match end {
            None => output.push(ensure_ascii(start)?),
            Some(end) => {
                output.push(ensure_ascii(start)?);
                output.push('-');
                output.push(ensure_ascii(end)?);
            }
        }
    }
    Ok(output)
}

#[test]
fn test_encode() {
    let test = |input, output: Result<&str, &str>| {
        let output = output.map(str::to_string).map_err(str::to_string);
        assert_eq!(encode(input), output);
    };
    test("", Ok(""));
    test("A", Ok("A"));
    test("é", Err("non-ASCII byte"));
    test("-", Ok("--"));
    test(",-.", Ok(",--."));
    test("+,-./", Ok("+-,--.-/"));
    test("AB", Ok("A-B"));
    test("ABCDEF", Ok("A-F"));
    test("ABCDEFabcdef", Ok("A-Fa-f"));
    test("ABCDEFabcdef-", Ok("A-Fa-f--"));
    test("ABCDEF-abcdef", Ok("A-F--a-f"));
    test("--", Ok("----"));
}

pub fn decode(input: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut input = input.as_bytes();
    while input.len() > 0 {
        if input.len() == 1 {
            if input[0] == b'-' {
                // -$  fail
                return Err("invalid trailing range".to_string());
            } else {
                // x$  ok(x)
            }
        } else if input[0] == b'-' {
            if input[1] != b'-' {
                // -x...  fail
                return Err("missing range start".to_string());
            }
            // --...  ok(--)
            input = &input[1 ..];
        } else if input[1] != b'-' {
            // xx...  ok(x)
        } else if input.len() == 2 {
            // x-$  fail
            return Err("missing range end".to_string());
        } else if input[2] == b'-' {
            // x--...  ok(x)
        } else {
            // x-x...  ok(x-x)
            let start = ensure_ascii(input[0])?;
            let end = ensure_ascii(input[2])?;
            if start > end {
                return Err("empty range".to_string());
            }
            for x in input[0] .. input[2] {
                output.push(x as char);
            }
            input = &input[2 ..];
        }
        output.push(ensure_ascii(input[0])?);
        input = &input[1 ..];
    }
    Ok(output)
}

#[test]
fn test_decode() {
    let test = |input, output: Result<&str, &str>| {
        let output = output.map(str::to_string).map_err(str::to_string);
        assert_eq!(decode(input), output);
    };
    test("", Ok(""));
    test("A", Ok("A"));
    test("é", Err("non-ASCII byte"));
    test("-", Err("invalid trailing range"));
    test("AB", Ok("AB"));
    test("A-", Err("missing range end"));
    test("-A", Err("missing range start"));
    test("--", Ok("-"));
    test("ABC", Ok("ABC"));
    test("AB-", Err("missing range end"));
    test("A-B", Ok("AB"));
    test("A-E", Ok("ABCDE"));
    test("A--", Ok("A-"));
    test("-AB", Err("missing range start"));
    test("-A-", Err("missing range start"));
    test("--A", Ok("-A"));
    test("---", Err("invalid trailing range"));
    test("A-E--a-e", Ok("ABCDE-abcde"));
    test("A-Ea-e--", Ok("ABCDEabcde-"));
    test("A-Ea-e--_", Ok("ABCDEabcde-_"));
    test("_--A-Ea-e", Ok("_-ABCDEabcde"));
    test(",-.", Ok(",-."));
    test("+-/", Ok("+,-./"));
}
