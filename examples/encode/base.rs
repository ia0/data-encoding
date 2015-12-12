use std;

use data_encoding::base;
use data_encoding::encode;
use data_encoding::decode;
use data_encoding::base2;
use data_encoding::base4;
use data_encoding::base8;
use data_encoding::base16;
use data_encoding::base32;
use data_encoding::base32hex;
use data_encoding::base64;
use data_encoding::base64url;

use error::Error;

pub trait Base {
    fn encode_len(&self, usize) -> usize;
    fn decode_len(&self, usize) -> usize;
    fn encode_mut(&self, &[u8], &mut [u8]);
    fn decode_mut(&self, &[u8], &mut [u8]) -> Result<usize, decode::Error>;
}

struct Official {
    encode_len: fn(usize) -> usize,
    decode_len: fn(usize) -> usize,
    encode_mut: fn(&[u8], &mut [u8]),
    decode_mut: fn(&[u8], &mut [u8]) -> Result<usize, decode::Error>,
}

impl Base for Official {
    fn encode_len(&self, len: usize) -> usize {
        (self.encode_len)(len)
    }
    fn decode_len(&self, len: usize) -> usize {
        (self.decode_len)(len)
    }
    fn encode_mut(&self, input: &[u8], output: &mut [u8]) {
        (self.encode_mut)(input, output)
    }
    fn decode_mut(&self, input: &[u8], output: &mut [u8]) -> Result<usize, decode::Error> {
        (self.decode_mut)(input, output)
    }
}

pub fn lookup(b: String) -> Result<Box<Base>, Error> {
    macro_rules! from {
        ($b: ident) => { Ok(Box::new(Official {
            encode_len: $b::encode_len,
            decode_len: $b::decode_len,
            encode_mut: $b::encode_mut,
            decode_mut: $b::decode_mut,
        })) };
    }
    match &b as &str {
        "2" => from!(base2),
        "4" => from!(base4),
        "8" => from!(base8),
        "16" | "hex" => from!(base16),
        "32" => from!(base32),
        "32hex" => from!(base32hex),
        "64" => from!(base64),
        "64url" => from!(base64url),
        _ => Ok(Box::new(try!(build(b).map_err(Error::BuildBase)))),
    }
}

struct Custom {
    val: Vec<u8>,
    sym: Vec<u8>,
    bit: u8,
    pad: u8,
}

impl Base for Custom {
    fn encode_len(&self, len: usize) -> usize {
        encode::encode_len(self, len)
    }
    fn decode_len(&self, len: usize) -> usize {
        decode::decode_len(self, len)
    }
    fn encode_mut(&self, input: &[u8], output: &mut [u8]) {
        encode::encode_mut(self, input, output)
    }
    fn decode_mut(&self, input: &[u8], output: &mut [u8]) -> Result<usize, decode::Error> {
        decode::decode_mut(self, input, output)
    }
}

impl base::Base for Custom {
    fn bit(&self) -> usize {
        self.bit as usize
    }

    fn pad(&self) -> u8 {
        self.pad
    }

    fn val(&self, x: u8) -> Option<u8> {
        let v = self.val[x as usize];
        if v < 128 { Some(v) } else { None }
    }

    fn sym(&self, x: u8) -> u8 {
        self.sym[x as usize]
    }
}

fn build(b: String) -> Result<Custom, BuildError> {
    let mut i = b.as_bytes().iter().cloned();
    let pad = try!(i.next().ok_or(BuildError::BadLength));
    let sym: Vec<_> = i.collect();
    let bit = {
        let mut b = 0;
        let mut n = sym.len();
        while n > 1 {
            n /= 2;
            b += 1;
        }
        b
    };
    check!(sym.len() == 1 << bit as usize, BuildError::BadLength);
    check!(1 <= bit && bit <= 6, BuildError::BadLength);
    let mut val = vec![128u8; 256];
    for (i, &s) in sym.iter().enumerate() {
        check!(val[s as usize] == 128, BuildError::Duplicate(s));
        val[s as usize] = i as u8;
    }
    let base = Custom { val: val, sym: sym, bit: bit, pad: pad };
    match base::valid(&base) {
        Ok(()) => Ok(base),
        Err(e) => Err(BuildError::BadBase(e)),
    }
}

#[derive(Debug)]
pub enum BuildError {
    BadLength,
    Duplicate(u8),
    BadBase(base::ValidError),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &BuildError::BadLength => write!(f, "Length must be 3, 5, 9, 17, 33, or 65."),
            &BuildError::Duplicate(s) => write!(f, "Duplicate symbol {:?}.", s as char),
            &BuildError::BadBase(ref e) => <std::fmt::Display>::fmt(e, f),
        }
    }
}

impl std::error::Error for BuildError {
    fn description(&self) -> &str {
        match self {
            &BuildError::BadLength => "invalid length",
            &BuildError::Duplicate(_) => "duplicate symbol",
            &BuildError::BadBase(_) => "invalid base",
        }
    }
}
