extern crate data_encoding;

use data_encoding::DecodeKind::*;
use data_encoding::{DecodeError, Encoding, Specification};

macro_rules! test {
    (fn $t: ident; $($s: stmt);*;) => {
        #[test]
        fn $t() {
            fn test(b: &Encoding, x: &[u8], y: &[u8]) {
                assert_eq!(&b.encode(x).into_bytes() as &[u8], y);
                assert_eq!(&b.decode(y).unwrap() as &[u8], x);
            }
            $($s)*
        }
    };
}

fn errmsg<T, E: std::error::Error>(x: Result<T, E>) -> String {
    x.err().unwrap().to_string()
}

test! {
    fn base2;
    let mut s = Specification::new();
    s.symbols.push_str("01");
    let b = s.encoding().unwrap();
    test(&b, b"", b"");
    test(&b, b"f", b"01100110");
    test(&b, b"fo", b"0110011001101111");
    test(&b, b"foo", b"011001100110111101101111");
}

test! {
    fn base4;
    let mut s = Specification::new();
    s.symbols.push_str("0123");
    let b = s.encoding().unwrap();
    test(&b, b"", b"");
    test(&b, b"f", b"1212");
    test(&b, b"fo", b"12121233");
    test(&b, b"foo", b"121212331233");
    test(&b, b"foob", b"1212123312331202");
    test(&b, b"fooba", b"12121233123312021201");
    test(&b, b"foobar", b"121212331233120212011302");
}

test! {
    fn base8;
    let mut s = Specification::new();
    s.symbols.push_str("01234567");
    s.padding = Some('=');
    let b = s.encoding().unwrap();
    test(&b, b"", b"");
    test(&b, b"f", b"314=====");
    test(&b, b"fo", b"314674==");
    test(&b, b"foo", b"31467557");
    test(&b, b"foob", b"31467557304=====");
    test(&b, b"fooba", b"31467557304604==");
    test(&b, b"foobar", b"3146755730460562");
}

test! {
    fn hexlower;
    let b = &data_encoding::HEXLOWER;
    test(b, b"", b"");
    test(b, b"f", b"66");
    test(b, b"fo", b"666f");
    test(b, b"foo", b"666f6f");
    test(b, b"foob", b"666f6f62");
    test(b, b"fooba", b"666f6f6261");
    test(b, b"foobar", b"666f6f626172");
    assert_eq!(data_encoding::HEXLOWER.decode(b"6f").unwrap(), b"o");
    assert!(data_encoding::HEXLOWER.decode(b"6F").is_err());
    assert_eq!(data_encoding::HEXLOWER_PERMISSIVE.decode(b"6F").unwrap(), b"o");
}

test! {
    fn hexupper;
    let b = &data_encoding::HEXUPPER;
    test(b, b"", b"");
    test(b, b"f", b"66");
    test(b, b"fo", b"666F");
    test(b, b"foo", b"666F6F");
    test(b, b"foob", b"666F6F62");
    test(b, b"fooba", b"666F6F6261");
    test(b, b"foobar", b"666F6F626172");
    assert_eq!(data_encoding::HEXUPPER.decode(b"6F").unwrap(), b"o");
    assert!(data_encoding::HEXUPPER.decode(b"6f").is_err());
    assert_eq!(data_encoding::HEXUPPER_PERMISSIVE.decode(b"6f").unwrap(), b"o");
}

test! {
    fn base32;
    let b = &data_encoding::BASE32;
    test(b, b"", b"");
    test(b, b"f", b"MY======");
    test(b, b"fo", b"MZXQ====");
    test(b, b"foo", b"MZXW6===");
    test(b, b"foob", b"MZXW6YQ=");
    test(b, b"fooba", b"MZXW6YTB");
    test(b, b"foobar", b"MZXW6YTBOI======");
}

test! {
    fn base32hex;
    let b = &data_encoding::BASE32HEX;
    test(b, b"", b"");
    test(b, b"f", b"CO======");
    test(b, b"fo", b"CPNG====");
    test(b, b"foo", b"CPNMU===");
    test(b, b"foob", b"CPNMUOG=");
    test(b, b"fooba", b"CPNMUOJ1");
    test(b, b"foobar", b"CPNMUOJ1E8======");
}

test! {
    fn base32_dnscurve;
    let b = &data_encoding::BASE32_DNSCURVE;
    test(b, &[0x64, 0x88], b"4321");
    test(b, b"f", b"63");
    test(b, b"fo", b"6vv0");
    test(b, b"foo", b"6vvy6");
    test(b, b"foob", b"6vvy6k1");
    test(b, b"fooba", b"6vvy6k5d");
    test(b, b"foobar", b"6vvy6k5dl3");
    assert_eq!(b.decode(b"6VVY6K5DL3").unwrap(), b"foobar");
    assert!(b.decode(b"4322").is_err());
    assert!(b.decode(b"4324").is_err());
    assert!(b.decode(b"4328").is_err());
    assert!(b.decode(b"432j").is_err());
    assert!(b.decode(b"08").is_err());
    assert!(b.decode(b"0000j").is_err());
    assert!(b.decode(b"0000004").is_err());
}

test! {
    fn base64;
    let b = &data_encoding::BASE64;
    test(b, b"", b"");
    test(b, b"f", b"Zg==");
    test(b, b"fo", b"Zm8=");
    test(b, b"foo", b"Zm9v");
    test(b, b"foob", b"Zm9vYg==");
    test(b, b"fooba", b"Zm9vYmE=");
    test(b, b"foobar", b"Zm9vYmFy");
    test(b, &[251u8, 240], b"+/A=");
}

test! {
    fn base64url;
    let b = &data_encoding::BASE64URL;
    test(b, b"", b"");
    test(b, b"f", b"Zg==");
    test(b, b"fo", b"Zm8=");
    test(b, b"foo", b"Zm9v");
    test(b, b"foob", b"Zm9vYg==");
    test(b, b"fooba", b"Zm9vYmE=");
    test(b, b"foobar", b"Zm9vYmFy");
    test(b, &[251u8, 240], b"-_A=");
}

test! {
    fn base64_no_pad;
    let b = &data_encoding::BASE64_NOPAD;
    test(&b, b"", b"");
    test(&b, b"f", b"Zg");
    test(&b, b"fo", b"Zm8");
    test(&b, b"foo", b"Zm9v");
    test(&b, b"foob", b"Zm9vYg");
    test(&b, b"fooba", b"Zm9vYmE");
    test(&b, b"foobar", b"Zm9vYmFy");
}

#[test]
fn base32_error() {
    let b = &data_encoding::BASE32;
    assert_eq!(b.decode(b"ABC").err().unwrap(), DecodeError { position: 0, kind: Length });
    assert_eq!(b.decode(b"========").err().unwrap(), DecodeError { position: 0, kind: Padding });
    assert_eq!(b.decode(b"MB======").err().unwrap(), DecodeError { position: 1, kind: Trailing });
    assert_eq!(b.decode(b"MA===AAA").err().unwrap(), DecodeError { position: 2, kind: Symbol });
    assert_eq!(b.decode(b"MAA=====").err().unwrap(), DecodeError { position: 3, kind: Padding });
    assert_eq!(b.decode(b"MAABBB==").err().unwrap(), DecodeError { position: 6, kind: Padding });
}

#[test]
fn base64_error() {
    let b = &data_encoding::BASE64;
    assert_eq!(b.decode(b"====").err().unwrap(), DecodeError { position: 0, kind: Padding });
    assert_eq!(b.decode(b"====").err().unwrap(), DecodeError { position: 0, kind: Padding });
    assert_eq!(
        b.decode(b"Zm9vYmFy====").err().unwrap(),
        DecodeError { position: 8, kind: Padding }
    );
    assert_eq!(
        b.decode(b"Zm9vYmFy====").err().unwrap(),
        DecodeError { position: 8, kind: Padding }
    );
    assert_eq!(b.decode(b"YmE=Zg==Zg==").unwrap(), b"baff");
    assert_eq!(b.decode(b"Zm9vYmFy----").err().unwrap(), DecodeError { position: 8, kind: Symbol });
    assert_eq!(b.decode(b"YmE=-mFyZg==").err().unwrap(), DecodeError { position: 4, kind: Symbol });
    assert_eq!(b.decode(b"YmE=-g==Zg==").err().unwrap(), DecodeError { position: 4, kind: Symbol });
    assert_eq!(b.decode(b"YmE=Z-==Zg==").err().unwrap(), DecodeError { position: 5, kind: Symbol });
    assert_eq!(b.decode(b"YmE=Y-FyZg==").err().unwrap(), DecodeError { position: 5, kind: Symbol });
    assert_eq!(
        b.decode(b"YmE=Z===Zg==").err().unwrap(),
        DecodeError { position: 5, kind: Padding }
    );
    assert_eq!(
        b.decode(b"YmE=Zh==Zg==").err().unwrap(),
        DecodeError { position: 5, kind: Trailing }
    );
    assert_eq!(b.decode_len(4).unwrap(), 3);
    assert_eq!(b.decode_len(5).err().unwrap(), DecodeError { position: 4, kind: Length });
    assert_eq!(b.decode_len(6).err().unwrap(), DecodeError { position: 4, kind: Length });
    assert_eq!(b.decode_len(7).err().unwrap(), DecodeError { position: 4, kind: Length });
}

#[test]
fn base64_nopad_error() {
    let mut s = data_encoding::BASE64.specification();
    s.padding = None;
    let b = s.encoding().unwrap();
    assert_eq!(b.decode(b"Z").err().unwrap(), DecodeError { position: 0, kind: Length });
    assert_eq!(b.decode(b"Zh").err().unwrap(), DecodeError { position: 1, kind: Trailing });
    assert_eq!(b.decode(b"Zg==").err().unwrap(), DecodeError { position: 2, kind: Symbol });
    assert_eq!(b.decode_len(4).unwrap(), 3);
    assert_eq!(b.decode_len(5).err().unwrap(), DecodeError { position: 4, kind: Length });
    assert_eq!(b.decode_len(6).unwrap(), 4);
    assert_eq!(b.decode_len(7).unwrap(), 5);
}

#[test]
fn bit_order() {
    let mut spec = Specification::new();
    spec.symbols.push_str("0123456789abcdef");
    let msb = spec.encoding().unwrap();
    spec.bit_order = data_encoding::BitOrder::LeastSignificantFirst;
    let lsb = spec.encoding().unwrap();
    assert_eq!(msb.encode(b"ABC"), "414243");
    assert_eq!(lsb.encode(b"ABC"), "142434");
}

#[test]
fn trailing_bits() {
    let mut spec = Specification::new();
    spec.symbols.push_str("01234567");
    let strict = spec.encoding().unwrap();
    spec.check_trailing_bits = false;
    let permissive = spec.encoding().unwrap();
    assert!(strict.decode(b"001").is_err());
    assert!(permissive.decode(b"001").is_ok());
}

#[test]
fn ignore() {
    fn skip(buf: &[u8], cmp: &mut [u8], shift: &mut [usize]) -> usize {
        let mut j = 0;
        for i in 0 .. buf.len() {
            if buf[i] == b' ' {
                continue;
            }
            cmp[j] = buf[i];
            shift[j] = i;
            j += 1;
        }
        j
    }
    fn check(base: &Encoding, buf: &[u8], cmp: &[u8], shift: &[usize]) {
        let res = base.decode(buf);
        match base.decode(cmp) {
            Ok(x) => assert_eq!(Ok(x), res),
            Err(mut x) => {
                x.position = shift[x.position];
                assert_eq!(Err(x), res);
            }
        }
    }
    fn incr(chars: &[u8], idx: &mut [usize], buf: &mut [u8]) -> bool {
        for i in 0 .. idx.len() {
            idx[i] += 1;
            if idx[i] == chars.len() {
                idx[i] = 0;
            }
            buf[i] = chars[idx[i]];
            if idx[i] > 0 {
                return true;
            }
        }
        false
    }
    fn forall(base: &Encoding, chars: &[u8], max: usize) {
        let mut idx = vec![0; max];
        let mut buf = vec![0; max];
        let mut cmp = vec![0; max];
        let mut shift = vec![0; max];
        for size in 0 .. (max + 1) {
            loop {
                let len = skip(&buf[.. size], &mut cmp, &mut shift);
                check(base, &buf[.. size], &cmp[.. len], &shift[.. len]);
                if !incr(chars, &mut idx[.. size], &mut buf[.. size]) {
                    break;
                }
            }
        }
    }
    let mut spec = Specification::new();
    spec.symbols.push_str("01234567");
    spec.ignore.push_str(" ");
    let no_pad = spec.encoding().unwrap();
    spec.padding = Some('=');
    let padded = spec.encoding().unwrap();
    if cfg!(debug_assertions) {
        forall(&no_pad, b"0 ", 14);
        forall(&no_pad, b"0 .", 9);
        forall(&padded, b"0= ", 9);
        forall(&padded, b"0= .", 7);
    } else {
        forall(&no_pad, b"0 ", 18);
        forall(&no_pad, b"0 .", 11);
        forall(&padded, b"0= ", 11);
        forall(&padded, b"0= .", 9);
    }
    assert_eq!(padded.decode(b"000=====").unwrap(), [0]);
    assert_eq!(padded.decode(b"000   =====").unwrap(), [0]);
    assert_eq!(padded.decode(b"000000==").unwrap(), [0, 0]);
    assert_eq!(
        padded.decode(b"000   ==").err().unwrap(),
        DecodeError { position: 0, kind: Length }
    );
}

#[test]
fn translate() {
    let mut spec = Specification::new();
    spec.symbols.push_str("01234567");
    spec.padding = Some('=');
    spec.ignore.push_str(" .");
    spec.translate.from.push_str("-_O");
    spec.translate.to.push_str("=.0");
    let base = spec.encoding().unwrap();
    assert_eq!(base.decode(b" O OO= =  = == ").unwrap(), [0]);
    assert_eq!(base.decode(b"O__OO__--_--.-").unwrap(), [0]);
    assert_eq!(base.decode(b"_____.  . . ...   ..").unwrap(), []);
}

#[test]
fn specification() {
    assert_eq!(errmsg(Specification::new().encoding()), "invalid number of symbols");
    let build = |sym, pad| {
        let mut spec = Specification::new();
        spec.symbols.push_str(sym);
        spec.padding = pad;
        spec.encoding()
    };
    assert_eq!(errmsg(build("é", None)), "non-ascii character");
    assert_eq!(errmsg(build("01", Some(' '))), "unnecessary padding");
    assert_eq!(errmsg(build("01234567", Some('é'))), "non-ascii character");
    assert_eq!(errmsg(build("01234567", Some('0'))), "'0' has conflicting definitions");
    assert_eq!(
        errmsg(build("0000000000000000000000000000000000000000000000000000000000000000", None)),
        "'0' has conflicting definitions"
    );
    let mut spec = Specification::new();
    spec.symbols.push_str("01");
    spec.translate.from.push_str("1");
    spec.translate.to.push_str("0");
    assert_eq!(errmsg(spec.encoding()), "'1' has conflicting definitions");
    spec.translate.from.push_str("12");
    assert_eq!(errmsg(spec.encoding()), "translate from/to length mismatch");
    spec.translate.from = "Z".to_string();
    spec.translate.to = "2".to_string();
    assert_eq!(errmsg(spec.encoding()), "'2' is undefined");
    let mut spec = Specification::new();
    spec.wrap.width = 1;
    spec.wrap.separator.push_str("\n");
    spec.symbols.push_str("01");
    assert_eq!(errmsg(spec.encoding()), "wrap width not a multiple of 8");
    spec.symbols.push_str("23");
    assert_eq!(errmsg(spec.encoding()), "wrap width not a multiple of 4");
    spec.symbols.push_str("4567");
    assert_eq!(errmsg(spec.encoding()), "wrap width not a multiple of 8");
    spec.symbols.push_str("89abcdef");
    assert_eq!(errmsg(spec.encoding()), "wrap width not a multiple of 2");
    spec.symbols.push_str("ghijklmnopqrstuv");
    assert_eq!(errmsg(spec.encoding()), "wrap width not a multiple of 8");
    spec.symbols.push_str("wxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_");
    assert_eq!(errmsg(spec.encoding()), "wrap width not a multiple of 4");
    spec.wrap.separator.clear();
    let previous_encoding = spec.encoding().unwrap();
    spec.wrap.width = 0;
    spec.wrap.separator.push_str("\n");
    assert_eq!(spec.encoding().unwrap(), previous_encoding);
    spec.wrap.width = 256;
    assert_eq!(errmsg(spec.encoding()), "invalid wrap width or separator length");
}

#[test]
fn round_trip() {
    let test = |e: Encoding| {
        assert_eq!(e.specification().encoding().unwrap(), e);
    };
    test(data_encoding::HEXLOWER);
    test(data_encoding::HEXLOWER_PERMISSIVE);
    test(data_encoding::HEXUPPER);
    test(data_encoding::HEXUPPER_PERMISSIVE);
    test(data_encoding::BASE32);
    test(data_encoding::BASE32_NOPAD);
    test(data_encoding::BASE32HEX);
    test(data_encoding::BASE32HEX_NOPAD);
    test(data_encoding::BASE32_DNSSEC);
    test(data_encoding::BASE32_DNSCURVE);
    test(data_encoding::BASE64);
    test(data_encoding::BASE64_NOPAD);
    test(data_encoding::BASE64_MIME);
    test(data_encoding::BASE64URL);
    test(data_encoding::BASE64URL_NOPAD);
}

#[test]
fn is_canonical() {
    fn test(expect: bool, update: &dyn Fn(&mut Specification)) {
        let mut spec = Specification::new();
        spec.symbols.push_str("01234567");
        update(&mut spec);
        assert_eq!(expect, spec.encoding().unwrap().is_canonical());
    }
    test(true, &|_| {});
    test(false, &|spec| {
        spec.check_trailing_bits = false;
    });
    test(false, &|spec| {
        spec.padding = Some('=');
    });
    test(false, &|spec| {
        spec.ignore.push(' ');
    });
    test(false, &|spec| {
        spec.translate.from.push('O');
        spec.translate.to.push('0');
    });
}

#[test]
fn decode_error() {
    let b = &data_encoding::BASE64;
    assert_eq!(errmsg(b.decode(b"A")), "invalid length at 0");
    assert_eq!(errmsg(b.decode(b"A.AA")), "invalid symbol at 1");
    assert_eq!(errmsg(b.decode(b"AAB=")), "non-zero trailing bits at 2");
    assert_eq!(errmsg(b.decode(b"A===")), "invalid padding length at 1");
}

#[test]
fn encode_base() {
    let mut spec = data_encoding::BASE64.specification();
    spec.padding = None;
    let b = spec.encoding().unwrap();
    assert_eq!(b.encode(b""), "");
    assert_eq!(b.encode(b"h"), "aA");
    assert_eq!(b.encode(b"he"), "aGU");
    assert_eq!(b.encode(b"hel"), "aGVs");
    assert_eq!(b.encode(b"hell"), "aGVsbA");
    assert_eq!(b.encode(b"hello"), "aGVsbG8");
}

#[test]
fn decode_base() {
    let mut spec = data_encoding::BASE64.specification();
    spec.padding = None;
    let b = spec.encoding().unwrap();
    let err_length = |pos| Err(DecodeError { position: pos, kind: Length });
    let err_symbol = |pos| Err(DecodeError { position: pos, kind: Symbol });
    let err_trailing = |pos| Err(DecodeError { position: pos, kind: Trailing });
    assert_eq!(b.decode(b""), Ok(vec![]));
    assert_eq!(b.decode(b"A"), err_length(0));
    assert_eq!(b.decode(b"AQ"), Ok(vec![1]));
    assert_eq!(b.decode(b"AQE"), Ok(vec![1; 2]));
    assert_eq!(b.decode(b"AQEB"), Ok(vec![1; 3]));
    assert_eq!(b.decode(b"AAAAA"), err_length(4));
    assert_eq!(b.decode(b"AQEBAQ"), Ok(vec![1; 4]));
    assert_eq!(b.decode(b"AQEBAQE"), Ok(vec![1; 5]));
    assert_eq!(b.decode(b"AQEBAQEB"), Ok(vec![1; 6]));
    assert_eq!(b.decode(b".A"), err_symbol(0));
    assert_eq!(b.decode(b"A."), err_symbol(1));
    assert_eq!(b.decode(b".AA"), err_symbol(0));
    assert_eq!(b.decode(b"A.A"), err_symbol(1));
    assert_eq!(b.decode(b"AA."), err_symbol(2));
    assert_eq!(b.decode(b".AAA"), err_symbol(0));
    assert_eq!(b.decode(b"A.AA"), err_symbol(1));
    assert_eq!(b.decode(b"AA.A"), err_symbol(2));
    assert_eq!(b.decode(b"AAA."), err_symbol(3));
    assert_eq!(b.decode(b"AB"), err_trailing(1));
    assert_eq!(b.decode(b"AAB"), err_trailing(2));
}

#[test]
fn encode_pad() {
    let b = &data_encoding::BASE64;
    assert_eq!(b.encode(b""), "");
    assert_eq!(b.encode(b"h"), "aA==");
    assert_eq!(b.encode(b"he"), "aGU=");
    assert_eq!(b.encode(b"hel"), "aGVs");
    assert_eq!(b.encode(b"hell"), "aGVsbA==");
    assert_eq!(b.encode(b"hello"), "aGVsbG8=");
}

#[test]
fn decode_pad() {
    let b = &data_encoding::BASE64;
    let err_length = |pos| Err(DecodeError { position: pos, kind: Length });
    let err_symbol = |pos| Err(DecodeError { position: pos, kind: Symbol });
    let err_trailing = |pos| Err(DecodeError { position: pos, kind: Trailing });
    let err_padding = |pos| Err(DecodeError { position: pos, kind: Padding });
    assert_eq!(b.decode(b""), Ok(vec![]));
    assert_eq!(b.decode(b"A"), err_length(0));
    assert_eq!(b.decode(b"AA"), err_length(0));
    assert_eq!(b.decode(b"AAA"), err_length(0));
    assert_eq!(b.decode(b"AQ=="), Ok(vec![1; 1]));
    assert_eq!(b.decode(b"AQE="), Ok(vec![1; 2]));
    assert_eq!(b.decode(b"AQEB"), Ok(vec![1; 3]));
    assert_eq!(b.decode(b"AAAAA"), err_length(4));
    assert_eq!(b.decode(b"AAAAAA"), err_length(4));
    assert_eq!(b.decode(b"AAAAAAA"), err_length(4));
    assert_eq!(b.decode(b"AQEBAQ=="), Ok(vec![1; 4]));
    assert_eq!(b.decode(b"AQEBAQE="), Ok(vec![1; 5]));
    assert_eq!(b.decode(b"AQEBAQEB"), Ok(vec![1; 6]));
    assert_eq!(b.decode(b"AQ==AQ=="), Ok(vec![1; 2]));
    assert_eq!(b.decode(b"AQE=AQ=="), Ok(vec![1; 3]));
    assert_eq!(b.decode(b"AQ==AQE="), Ok(vec![1; 3]));
    assert_eq!(b.decode(b"AQEBAQ=="), Ok(vec![1; 4]));
    assert_eq!(b.decode(b"AQE=AQE="), Ok(vec![1; 4]));
    assert_eq!(b.decode(b"AQ==AQEB"), Ok(vec![1; 4]));
    assert_eq!(b.decode(b"AQEBAQE="), Ok(vec![1; 5]));
    assert_eq!(b.decode(b"AQE=AQEB"), Ok(vec![1; 5]));
    assert_eq!(b.decode(b".A=="), err_symbol(0));
    assert_eq!(b.decode(b"A.=="), err_symbol(1));
    assert_eq!(b.decode(b"A=.="), err_symbol(1));
    assert_eq!(b.decode(b"A==."), err_symbol(1));
    assert_eq!(b.decode(b".AA="), err_symbol(0));
    assert_eq!(b.decode(b"A.A="), err_symbol(1));
    assert_eq!(b.decode(b"AA.="), err_symbol(2));
    assert_eq!(b.decode(b"AA=."), err_symbol(2));
    assert_eq!(b.decode(b".AAA"), err_symbol(0));
    assert_eq!(b.decode(b"A.AA"), err_symbol(1));
    assert_eq!(b.decode(b"AA.A"), err_symbol(2));
    assert_eq!(b.decode(b"AAA."), err_symbol(3));
    assert_eq!(b.decode(b"A==="), err_padding(1));
    assert_eq!(b.decode(b"===="), err_padding(0));
    assert_eq!(b.decode(b"AB=="), err_trailing(1));
    assert_eq!(b.decode(b"AAB="), err_trailing(2));
}

#[test]
fn encode_wrap() {
    let mut spec = data_encoding::BASE64.specification();
    spec.padding = None;
    spec.wrap.width = 4;
    spec.wrap.separator.push_str(":");
    let b = spec.encoding().unwrap();
    assert_eq!(b.encode(b""), "");
    assert_eq!(b.encode(b"h"), "aA:");
    assert_eq!(b.encode(b"he"), "aGU:");
    assert_eq!(b.encode(b"hel"), "aGVs:");
    assert_eq!(b.encode(b"hell"), "aGVs:bA:");
    assert_eq!(b.encode(b"hello"), "aGVs:bG8:");
}

#[test]
fn decode_wrap() {
    let mut spec = data_encoding::BASE64.specification();
    spec.padding = None;
    spec.ignore.push_str(":");
    let b = spec.encoding().unwrap();
    let err_length = |pos| Err(DecodeError { position: pos, kind: Length });
    let err_symbol = |pos| Err(DecodeError { position: pos, kind: Symbol });
    let err_trailing = |pos| Err(DecodeError { position: pos, kind: Trailing });
    assert_eq!(b.decode(b":"), Ok(vec![]));
    assert_eq!(b.decode(b":A:"), err_length(1));
    assert_eq!(b.decode(b":A:Q:"), Ok(vec![1]));
    assert_eq!(b.decode(b":A:Q:E:"), Ok(vec![1; 2]));
    assert_eq!(b.decode(b":A:Q:E:B:"), Ok(vec![1; 3]));
    assert_eq!(b.decode(b":A:A:A:A:A:"), err_length(9));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:"), Ok(vec![1; 4]));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:E:"), Ok(vec![1; 5]));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:E:B:"), Ok(vec![1; 6]));
    assert_eq!(b.decode(b":.:A:"), err_symbol(1));
    assert_eq!(b.decode(b":A:.:"), err_symbol(3));
    assert_eq!(b.decode(b":.:A:A:"), err_symbol(1));
    assert_eq!(b.decode(b":A:.:A:"), err_symbol(3));
    assert_eq!(b.decode(b":A:A:.:"), err_symbol(5));
    assert_eq!(b.decode(b":.:A:A:A:"), err_symbol(1));
    assert_eq!(b.decode(b":A:.:A:A:"), err_symbol(3));
    assert_eq!(b.decode(b":A:A:.:A:"), err_symbol(5));
    assert_eq!(b.decode(b":A:A:A:.:"), err_symbol(7));
    assert_eq!(b.decode(b":A:B:"), err_trailing(3));
    assert_eq!(b.decode(b":A:A:B:"), err_trailing(5));
}

#[test]
fn encode_pad_wrap() {
    let mut spec = data_encoding::BASE64.specification();
    spec.wrap.width = 4;
    spec.wrap.separator.push_str(":");
    let b = spec.encoding().unwrap();
    assert_eq!(b.encode(b""), "");
    assert_eq!(b.encode(b"h"), "aA==:");
    assert_eq!(b.encode(b"he"), "aGU=:");
    assert_eq!(b.encode(b"hel"), "aGVs:");
    assert_eq!(b.encode(b"hell"), "aGVs:bA==:");
    assert_eq!(b.encode(b"hello"), "aGVs:bG8=:");
}

#[test]
fn decode_pad_wrap() {
    let mut spec = data_encoding::BASE64.specification();
    spec.ignore.push_str(":");
    let b = spec.encoding().unwrap();
    let err_length = |pos| Err(DecodeError { position: pos, kind: Length });
    let err_symbol = |pos| Err(DecodeError { position: pos, kind: Symbol });
    let err_trailing = |pos| Err(DecodeError { position: pos, kind: Trailing });
    let err_padding = |pos| Err(DecodeError { position: pos, kind: Padding });
    assert_eq!(b.decode(b":"), Ok(vec![]));
    assert_eq!(b.decode(b":A:"), err_length(1));
    assert_eq!(b.decode(b":A:A:"), err_length(1));
    assert_eq!(b.decode(b":A:A:A:"), err_length(1));
    assert_eq!(b.decode(b":A:Q:=:=:"), Ok(vec![1; 1]));
    assert_eq!(b.decode(b":A:Q:E:=:"), Ok(vec![1; 2]));
    assert_eq!(b.decode(b":A:Q:E:B:"), Ok(vec![1; 3]));
    assert_eq!(b.decode(b":A:A:A:A:A:"), err_length(9));
    assert_eq!(b.decode(b":A:A:A:A:A:A:"), err_length(9));
    assert_eq!(b.decode(b":A:A:A:A:A:A:A:"), err_length(9));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:=:=:"), Ok(vec![1; 4]));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:E:=:"), Ok(vec![1; 5]));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:E:B:"), Ok(vec![1; 6]));
    assert_eq!(b.decode(b":A:Q:=:=:A:Q:=:=:"), Ok(vec![1; 2]));
    assert_eq!(b.decode(b":A:Q:E:=:A:Q:=:=:"), Ok(vec![1; 3]));
    assert_eq!(b.decode(b":A:Q:=:=:A:Q:E:=:"), Ok(vec![1; 3]));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:=:=:"), Ok(vec![1; 4]));
    assert_eq!(b.decode(b":A:Q:E:=:A:Q:E:=:"), Ok(vec![1; 4]));
    assert_eq!(b.decode(b":A:Q:=:=:A:Q:E:B:"), Ok(vec![1; 4]));
    assert_eq!(b.decode(b":A:Q:E:B:A:Q:E:=:"), Ok(vec![1; 5]));
    assert_eq!(b.decode(b":A:Q:E:=:A:Q:E:B:"), Ok(vec![1; 5]));
    assert_eq!(b.decode(b":.:A:=:=:"), err_symbol(1));
    assert_eq!(b.decode(b":A:.:=:=:"), err_symbol(3));
    assert_eq!(b.decode(b":A:=:.:=:"), err_symbol(3));
    assert_eq!(b.decode(b":A:=:=:.:"), err_symbol(3));
    assert_eq!(b.decode(b":.:A:A:=:"), err_symbol(1));
    assert_eq!(b.decode(b":A:.:A:=:"), err_symbol(3));
    assert_eq!(b.decode(b":A:A:.:=:"), err_symbol(5));
    assert_eq!(b.decode(b":A:A:=:.:"), err_symbol(5));
    assert_eq!(b.decode(b":.:A:A:A:"), err_symbol(1));
    assert_eq!(b.decode(b":A:.:A:A:"), err_symbol(3));
    assert_eq!(b.decode(b":A:A:.:A:"), err_symbol(5));
    assert_eq!(b.decode(b":A:A:A:.:"), err_symbol(7));
    assert_eq!(b.decode(b":A:=:=:=:"), err_padding(3));
    assert_eq!(b.decode(b":=:=:=:=:"), err_padding(1));
    assert_eq!(b.decode(b":A:B:=:=:"), err_trailing(3));
    assert_eq!(b.decode(b":A:A:B:=:"), err_trailing(5));
}

#[test]
fn encode_append() {
    fn test(input: &[u8], output: &str, expected: &str) {
        let mut output = output.to_string();
        data_encoding::BASE64.encode_append(input, &mut output);
        assert_eq!(output, expected);
    }
    test(b"", "", "");
    test(b"foo", "", "Zm9v");
    test(b"foo", "bar", "barZm9v");
    test(b"fo", "", "Zm8=");
    test(b"fo", "ba", "baZm8=");
}
