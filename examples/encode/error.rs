use std::{error, fmt, io, num};

use data_encoding::decode;

#[derive(Debug)]
pub enum Error {
    ParseOpts(::getopts::Fail),
    ExtraArgs,
    BuildBase(::base::BuildError),
    Open(String, io::Error),
    Create(String, io::Error),
    WrapDecode,
    Decode(decode::Error),
    SkipEncode,
    ParseWrap(num::ParseIntError),
    Read(io::Error),
    Write(io::Error),
    ExtraInput,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match self {
            &ParseOpts(ref e) => e.fmt(f),
            &ExtraArgs => write!(f, "Extra arguments given."),
            &BuildBase(ref e) => e.fmt(f),
            &Open(ref p, ref e) => write!(f, "{}: {}", p, e),
            &Create(ref p, ref e) => write!(f, "{}: {}", p, e),
            &WrapDecode => write!(f, "Option --wrap is incompatible with --decode."),
            &Decode(ref e) => e.fmt(f),
            &SkipEncode => write!(f, "Option --skip is only valid with --decode."),
            &ParseWrap(_) => write!(f, "Option --wrap expects a non-negative number"),
            &Read(ref e) => write!(f, "Read error: {}", e),
            &Write(ref e) => write!(f, "Write error: {}", e),
            &ExtraInput => write!(f, "Decoding ended before end-of-file."),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match self {
            &ParseOpts(ref e) => e.description(),
            &ExtraArgs => "unexpected argument",
            &BuildBase(ref e) => e.description(),
            &Open(_, ref e) => e.description(),
            &Create(_, ref e) => e.description(),
            &WrapDecode => "option --wrap is incompatible with --decode",
            &Decode(ref e) => e.description(),
            &SkipEncode => "option --skip is only valid with --decode",
            &ParseWrap(ref e) => e.description(),
            &Read(ref e) => e.description(),
            &Write(ref e) => e.description(),
            &ExtraInput => "unexpected trailing input",
        }
    }
}
