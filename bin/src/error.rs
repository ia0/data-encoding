#[derive(Debug)]
pub enum Error {
    ParseOpts(::getopts::Fail),
    ExtraArgs(Vec<String>),
    UnexpectedCustom,
    MissingSymbols,
    InvalidBase,
    InvalidMode,
    InvalidTranslate,
    Decode(::data_encoding::DecodeError),
    Builder(::data_encoding::BuilderError),
    Open(String, ::std::io::Error),
    Create(String, ::std::io::Error),
    ParseBlock,
    ParseWrap,
    ParsePadding,
    Read(::std::io::Error),
    Write(::std::io::Error),
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::Error::*;
        match self {
            &ParseOpts(ref e) => e.fmt(f),
            &ExtraArgs(ref a) => write!(f, "Unexpected arguments {:?}", a),
            &UnexpectedCustom => write!(f, "Unexpected custom base options"),
            &MissingSymbols => write!(f, "Missing symbols for custom base"),
            &InvalidBase => write!(f, "Invalid base"),
            &InvalidMode => write!(f, "Invalid mode"),
            &InvalidTranslate => write!(f, "Translate length is odd"),
            &Decode(ref e) => e.fmt(f),
            &Builder(ref e) => e.fmt(f),
            &Open(ref p, ref e) => write!(f, "{}: {}", p, e),
            &Create(ref p, ref e) => write!(f, "{}: {}", p, e),
            &ParseBlock => write!(f, "Invalid block value"),
            &ParseWrap => write!(f, "Invalid wrap value"),
            &ParsePadding => write!(f, "Invalid padding value"),
            &Read(ref e) => write!(f, "Read error: {}", e),
            &Write(ref e) => write!(f, "Write error: {}", e),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match self {
            &ParseOpts(ref e) => e.description(),
            &ExtraArgs(_) => "unexpected argument",
            &UnexpectedCustom => "unexpected custom base options",
            &MissingSymbols => "missing symbols for custom base",
            &InvalidBase => "invalid base",
            &InvalidMode => "invalid mode",
            &InvalidTranslate => "invalid translate",
            &Decode(ref e) => e.description(),
            &Builder(ref e) => e.description(),
            &Open(_, ref e) => e.description(),
            &Create(_, ref e) => e.description(),
            &ParseBlock => "invalid block value",
            &ParseWrap => "invalid wrap value",
            &ParsePadding => "invalid padding value",
            &Read(ref e) => e.description(),
            &Write(ref e) => e.description(),
        }
    }
}
