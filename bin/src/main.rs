#![warn(unused_results)]

extern crate data_encoding;
extern crate getopts;

use getopts::Options;
use std::fs::File;
use std::io::Write;

macro_rules! check {
    ($e: expr, $c: expr) => {
        if !$c {
            return Err($e);
        }
    };
}

mod base;
mod error;
mod io;

use base::Base;
use error::Error;
use io::{ReadDelta, Skip, Wrap};

type Result<T> = std::result::Result<T, Error>;

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
        .reqopt("", "mode", "{encode|decode[_concat]|info}", "<mode>")
        .reqopt("", "base", "{16|hex|{32|32hex|64|64url}[_nopad]|custom}",
                "<base>")
        .optopt("", "input", "uses <file> instead of standard input", "<file>")
        .optopt("", "output", "uses <file> instead of standard output",
                "<file>")
        .optopt("", "block", "reads blocks of <size> bytes", "<size>")
        .optflag("", "skip", "when decoding, skips newlines")
        .optopt("", "wrap", "when encoding, wraps every <cols> characters",
                "<cols>")
        .optopt("", "symbols", "custom base uses <symbols>", "<symbols>")
        .optopt("", "padding", "custom base pads with <padding>", "<padding>")
        .optopt("", "translate",
                "when decoding, custom base translates <new> as <old>",
                "<new><old>")
        .optflag("", "ignore_trailing_bits",
                 "when decoding, custom base ignores trailing bits")
        .optflag("", "least_significant_bit_first",
                 "custom base bit-order is least significant bit first");

    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        let brief = format!("Usage: {} [<options>]", program);
        print!("{0}
Examples:
    # Encode using the RFC4648 base64 encoding
    {1} --mode=encode --base=64        # with padding
    {1} --mode=encode --base=64_nopad  # without padding

    # Show base information for the permissive hex encoding
    {1} --mode=info --base=hex

    # Decode using a custom hexadecimal encoding
    {1} --mode=decode --base=custom --symbols=0123456789abcdef \\
        --translate=ABCDEFabcdef

    # Decode using the DNSCurve base32 encoding
    {1} --mode=decode --base=custom \\
        --symbols=0123456789bcdfghjklmnpqrstuvwxyz \\
        --translate=BCDFGHJKLMNPQRSTUVWXYZbcdfghjklmnpqrstuvwxyz \\
        --least_significant_bit_first
", opts.usage(&brief), program);
        return Ok(());
    }

    let matches = opts.parse(&args).map_err(Error::ParseOpts)?;
    check!(Error::ExtraArgs(matches.free), matches.free.is_empty());
    let custom_options = ["symbols", "padding", "translate",
                          "ignore_trailing_bits", "least_significant_bit_first"]
        .iter().map(|s| String::from(*s)).collect::<Vec<String>>();
    check!(Error::UnexpectedCustom,
           matches.opt_str("base").unwrap().as_str() == "custom" ||
           !matches.opts_present(&custom_options));

    let mut base = match matches.opt_str("base").unwrap().as_str() {
        "custom" => match matches.opt_str("symbols") {
            None => return Err(Error::MissingSymbols),
            Some(symbols) => base::create(
                symbols, matches.opt_str("padding"),
                matches.opt_str("translate"),
                matches.opt_present("ignore_trailing_bits"),
                matches.opt_present("least_significant_bit_first"))?,
        },
        "16" => Base::NoPad { base: *::data_encoding::HEXUPPER },
        "hex" => Base::NoPad { base: *::data_encoding::HEXLOWER_PERMISSIVE },
        "32" => Base::Padded { concat: false, base: *::data_encoding::BASE32 },
        "32_nopad" => Base::NoPad { base: *::data_encoding::BASE32.no_pad() },
        "32hex" =>
            Base::Padded { concat: false, base: *::data_encoding::BASE32HEX },
        "32hex_nopad" =>
            Base::NoPad { base: *::data_encoding::BASE32HEX.no_pad() },
        "64" => Base::Padded { concat: false, base: *::data_encoding::BASE64 },
        "64_nopad" => Base::NoPad { base: *::data_encoding::BASE64.no_pad() },
        "64url" =>
            Base::Padded { concat: false, base: *::data_encoding::BASE64URL },
        "64url_nopad" =>
            Base::NoPad { base: *::data_encoding::BASE64URL.no_pad() },
        _ => return Err(Error::InvalidBase),
    };

    let encode = match matches.opt_str("mode").unwrap().as_str() {
        "encode" => true,
        "decode" => false,
        "decode_concat" => { base.concat()?; false },
        "info" => { base.info(); return Ok(()) },
        _ => return Err(Error::InvalidMode),
    };

    let mut input: Box<ReadDelta>;
    if let Some(file) = matches.opt_str("input") {
        input = Box::new(File::open(&file).map_err(|e| Error::Open(file, e))?);
    } else {
        input = Box::new(std::io::stdin());
    };

    let mut output: Box<Write>;
    if let Some(file) = matches.opt_str("output") {
        output = Box::new(File::create(&file)
                          .map_err(|e| Error::Create(file, e))?);
    } else {
        // TODO: Change the following lines when Stdout does not go
        // through a LineWriter anymore.
        output = Box::new(
            unsafe { <File as std::os::unix::io::FromRawFd>::from_raw_fd(1) });
        // output = Box::new(std::io::stdout());
    }
    output = Box::new(std::io::BufWriter::new(output));

    let size = matches.opt_str("block").unwrap_or("15360".to_owned())
        .parse().map_err(|_| Error::ParseBlock)?;
    check!(Error::ParseBlock, size >= 8);

    if encode {
        match matches.opt_str("wrap") {
            None => (),
            Some(wrap) => {
                let cols = wrap.parse().map_err(|_| Error::ParseWrap)?;
                check!(Error::ParseWrap, cols > 0);
                output = Box::new(Wrap::new(output, cols));
            },
        }
        io::encode(base, input, output, size)
    } else {
        if matches.opt_present("skip") {
            input = Box::new(Skip::new(input));
        }
        io::decode(base, input, output, size)
    }
}
