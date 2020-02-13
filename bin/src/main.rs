#![warn(unused_results)]

extern crate data_encoding;
extern crate getopts;

use data_encoding::{DecodeKind, Encoding};
use getopts::Options;
use std::fs::File;
use std::io::{Read, Write};

macro_rules! check {
    ($e: expr, $c: expr) => {
        if !$c {
            return Err($e);
        }
    };
}

#[derive(Debug)]
pub enum Error {
    ParseOpts(::getopts::Fail),
    ExtraArgs(Vec<String>),
    Cmdline(String),
    Decode(::data_encoding::DecodeError),
    Builder(::data_encoding::SpecificationError),
    IO(String, ::std::io::Error),
    Read(::std::io::Error),
    Write(::std::io::Error),
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::Error::*;
        match self {
            ParseOpts(ref e) => e.fmt(f),
            ExtraArgs(ref a) => write!(f, "Unexpected arguments {:?}", a),
            Cmdline(ref m) => write!(f, "{}", m),
            Decode(ref e) => e.fmt(f),
            Builder(ref e) => e.fmt(f),
            IO(ref p, ref e) => write!(f, "{}: {}", p, e),
            Read(ref e) => write!(f, "Read error: {}", e),
            Write(ref e) => write!(f, "Write error: {}", e),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

fn floor(x: usize, d: usize) -> usize {
    x / d * d
}
fn ceil(x: usize, d: usize) -> usize {
    floor(x + d - 1, d)
}

#[test]
fn floor_ceil() {
    assert_eq!(floor(10, 5), 10);
    assert_eq!(floor(13, 5), 10);
    assert_eq!(floor(15, 5), 15);
    assert_eq!(ceil(10, 5), 10);
    assert_eq!(ceil(13, 5), 15);
    assert_eq!(ceil(15, 5), 15);
}

fn encode_block(base: &Encoding) -> usize {
    match base.bit_width() {
        1 | 2 | 4 => 1,
        3 | 6 => 3,
        5 => 5,
        _ => unreachable!(),
    }
}
fn decode_block(base: &Encoding) -> usize {
    encode_block(base) * 8 / base.bit_width()
}

pub fn encode<R: Read, W: Write>(
    base: &Encoding, wrap: usize, mut reader: R, mut writer: W, size: usize,
) -> Result<()> {
    let block = if wrap == 0 {
        encode_block(&base)
    } else {
        assert_eq!(wrap * base.bit_width() % 8, 0);
        wrap * base.bit_width() / 8
    };
    assert_eq!(block % encode_block(&base), 0);
    assert!(size >= block);
    let mut input = vec![0u8; size];
    let mut output = vec![0u8; base.encode_len(size)];
    let mut rest = 0;
    loop {
        let ilen = reader.read(&mut input[rest ..]).map_err(Error::Read)?;
        let next = if ilen == 0 { rest } else { floor(rest + ilen, block) };
        let olen = base.encode_len(next);
        base.encode_mut(&input[0 .. next], &mut output[0 .. olen]);
        writer.write_all(&output[0 .. olen]).map_err(Error::Write)?;
        if ilen == 0 {
            return Ok(());
        }
        rest = rest + ilen - next;
        for i in 0 .. rest {
            input[i] = input[next + i];
        }
    }
}

pub fn decode<R, W>(base: &Encoding, mut reader: R, mut writer: W, size: usize) -> Result<()>
where
    R: Read,
    W: Write,
{
    let block = decode_block(base);
    assert!(size >= block);
    let mut input = vec![0u8; size];
    let mut output = vec![0u8; base.decode_len(ceil(size, block)).unwrap()];
    let mut pos = 0;
    let mut rest = 0;
    loop {
        let ilen = reader.read(&mut input[rest ..]).map_err(Error::Read)?;
        let next = if ilen == 0 { rest } else { floor(rest + ilen, block) };
        let mlen = base.decode_len(next).map_err(|mut error| {
            error.position += pos;
            Error::Decode(error)
        })?;
        let (next, olen) = match base.decode_mut(&input[0 .. next], &mut output[0 .. mlen]) {
            Ok(olen) => (next, olen),
            Err(mut partial) => {
                if partial.error.kind != DecodeKind::Length {
                    partial.error.position += pos;
                    return Err(Error::Decode(partial.error));
                }
                assert_ne!(ilen, 0);
                (partial.read, partial.written)
            }
        };
        writer.write_all(&output[0 .. olen]).map_err(Error::Write)?;
        rest = rest + ilen - next;
        if ilen == 0 {
            return Ok(());
        }
        for i in 0 .. rest {
            input[i] = input[next + i];
        }
        pos += next;
    }
}

fn main() {
    let (program, args): (_, Vec<_>) = {
        let mut i = std::env::args();
        match i.next() {
            None => ("data-encoding".into(), vec![]),
            Some(p) => (p, i.collect()),
        }
    };
    let program = program.rsplit('/').next().unwrap_or(&program);

    if let Err(e) = wrapped_main(program, args) {
        let _ = writeln!(&mut std::io::stderr(), "{}: {}", program, e);
        std::process::exit(1)
    }
}

fn wrapped_main(program: &str, args: Vec<String>) -> Result<()> {
    let mut opts = Options::new();
    let _ = opts
        .reqopt("m", "mode", "{encode|decode|describe}", "<mode>")
        .optopt("b", "base", "{16|hex|32|32hex|64|64url}", "<base>")
        .optopt("i", "input", "read from <file> instead of standard input", "<file>")
        .optopt("o", "output", "write to <file> instead of standard output", "<file>")
        .optopt("", "block", "read blocks of about <size> bytes", "<size>")
        .optopt("p", "padding", "pad with <padding>", "<padding>")
        .optopt("g", "ignore", "when decoding, ignore characters in <ignore>", "<ignore>")
        .optopt("w", "width", "when encoding, wrap every <cols> characters", "<cols>")
        .optopt("s", "separator", "when encoding, wrap with <separator>", "<separator>")
        .optopt("", "symbols", "define a custom base using <symbols>", "<symbols>")
        .optopt("", "translate", "when decoding, translate <new> as <old>", "<new><old>")
        .optflag("", "ignore_trailing_bits", "when decoding, ignore non-zero trailing bits")
        .optflag("", "least_significant_bit_first", "use least significant bit first bit-order");

    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        let brief = format!(
            r"Usage: {program} --mode=<mode> --base=<base> [<options>]
Usage: {program} --mode=<mode> --symbols=<symbols> [<options>]",
            program = program
        );
        print!(
            "{0}
Examples:
    # Encode using the RFC4648 base64 encoding
    {1} -mencode -b64     # without padding
    {1} -mencode -b64 -p= # with padding

    # Encode using the MIME base64 encoding
    {1} -mencode -b64 -p= -w76 -s$'\\r\\n'

    # Show base information for the permissive hexadecimal encoding
    {1} --mode=describe --base=hex

    # Decode using the DNSCurve base32 encoding
    {1} -mdecode \\
        --symbols=0123456789bcdfghjklmnpqrstuvwxyz \\
        --translate=BCDFGHJKLMNPQRSTUVWXYZbcdfghjklmnpqrstuvwxyz \\
        --least_significant_bit_first
",
            opts.usage(&brief),
            program
        );
        return Ok(());
    }

    let matches = opts.parse(&args).map_err(Error::ParseOpts)?;
    check!(Error::ExtraArgs(matches.free), matches.free.is_empty());

    let mut spec = match matches.opt_str("base") {
        None => {
            let mut spec = ::data_encoding::Specification::new();
            spec.symbols = matches
                .opt_str("symbols")
                .ok_or_else(|| Error::Cmdline("Base or symbols must be provided".into()))?;
            spec
        }
        Some(base) => {
            check!(
                Error::Cmdline("Base and symbols are incompatible".into()),
                !matches.opt_present("symbols")
            );
            match base.as_str() {
                "16" => ::data_encoding::HEXUPPER.specification(),
                "hex" => ::data_encoding::HEXLOWER_PERMISSIVE.specification(),
                "32" => ::data_encoding::BASE32.specification(),
                "32hex" => ::data_encoding::BASE32HEX.specification(),
                "64" => ::data_encoding::BASE64.specification(),
                "64url" => ::data_encoding::BASE64URL.specification(),
                _ => return Err(Error::Cmdline("Invalid base".into())),
            }
        }
    };
    if let Some(padding) = matches.opt_str("padding") {
        let mut chars = padding.chars();
        spec.padding = chars.next();
        check!(Error::Cmdline("Empty padding".into()), spec.padding.is_some());
        check!(Error::Cmdline("Padding must be a character".into()), chars.next().is_none());
    } else {
        spec.padding = None;
    }
    if let Some(mut newold) = matches.opt_str("translate") {
        let invalid_translate = Error::Cmdline("Invalid translate".into());
        check!(invalid_translate, newold.len() % 2 == 0);
        let len = newold.len() / 2;
        check!(invalid_translate, newold.is_char_boundary(len));
        spec.translate.to = newold.split_off(len);
        spec.translate.from = newold;
    }
    if matches.opt_present("ignore_trailing_bits") {
        spec.check_trailing_bits = false;
    }
    if matches.opt_present("least_significant_bit_first") {
        spec.bit_order = ::data_encoding::BitOrder::LeastSignificantFirst;
    }
    if let Some(ignore) = matches.opt_str("ignore") {
        spec.ignore.push_str(ignore.as_str());
    }
    if let Some(width) = matches.opt_str("width") {
        spec.wrap.width =
            width.parse().map_err(|_| Error::Cmdline("Invalid width value".into()))?;
    }
    if let Some(separator) = matches.opt_str("separator") {
        spec.wrap.separator.push_str(separator.as_str());
    }
    let base = spec.encoding().map_err(Error::Builder)?;

    let mode = match matches.opt_str("mode").unwrap().as_str() {
        "encode" => true,
        "decode" => false,
        "describe" => {
            println!("{:#?}", base.specification());
            return Ok(());
        }
        _ => return Err(Error::Cmdline("Invalid mode".into())),
    };

    let input: Box<dyn Read>;
    if let Some(file) = matches.opt_str("input") {
        input = Box::new(File::open(&file).map_err(|e| Error::IO(file, e))?);
    } else {
        input = Box::new(std::io::stdin());
    };

    let mut output: Box<dyn Write>;
    if let Some(file) = matches.opt_str("output") {
        output = Box::new(File::create(&file).map_err(|e| Error::IO(file, e))?);
    } else {
        output = stdout();
    }
    output = Box::new(std::io::BufWriter::new(output));

    let size = matches
        .opt_str("block")
        .unwrap_or_else(|| "15360".to_owned())
        .parse()
        .map_err(|_| Error::Cmdline("Invalid block value".into()))?;
    check!(Error::Cmdline("Block value must be greater or equal than 8".into()), size >= 8);

    if mode {
        encode(&base, spec.wrap.width, input, output, size)
    } else {
        decode(&base, input, output, size)
    }
}

// TODO: Change (and inline) the following lines when Stdout does not
// go through a LineWriter anymore.
#[cfg(target_os = "linux")]
fn stdout() -> Box<dyn Write> {
    Box::new(unsafe { <File as std::os::unix::io::FromRawFd>::from_raw_fd(1) })
}
#[cfg(not(target_os = "linux"))]
fn stdout() -> Box<dyn Write> {
    Box::new(std::io::stdout())
}
