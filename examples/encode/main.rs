#![warn(unused_results)]

// TODO: Test if we really need custom, or if it's not easier and
// faster to "tr | baseN | tr".

// TODO: Instead of --base <base> and --decode, maybe use --from <base> and --to <base>
// With <base> in 2, 4, 8, 16, hex, 32, 32hex, 64, 64url, raw, <custom>
//      <custom> is <pad><symbols>

extern crate getopts;
extern crate data_encoding;

use std::fs::File;
use std::io::Write;

use getopts::Options;

#[macro_use]
mod tool;
mod base;
mod error;
mod io;

use base::{Base, lookup};
use error::Error;
use io::{ReadShift, Operation, Skip, Wrap, repeat};

fn main() {
    let (program, args): (_, Vec<_>) = {
        let mut i = std::env::args();
        match i.next() {
            None => ("encode".into(), vec![]),
            Some(p) => (p, i.collect()),
        }
    };
    let program = program.rsplit('/').next().unwrap_or(&program);

    if let Err(e) = wrapped_main(program, args) {
        let _ = writeln!(&mut std::io::stderr(), "{}: {}", program, e);
        std::process::exit(1)
    }
}

fn wrapped_main(program: &str, args: Vec<String>) -> Result<(), Error> {
    // Define options.
    let mut opts = Options::new();
    let opts = opts
        .optopt("b", "base", "select base 2, 4, 8, 16 (or hex), 32, 32hex, 64, or 64url if <name> matches. Otherwise, build base using the first character of <name> as padding and the remaining characters as symbols in value order. Default is 64.", "<name>")
        .optflag("d", "decode", "decode data. Default is to encode data.")
        .optopt("i", "input", "use <file> as input. Default is to use standard input.", "<file>")
        .optopt("o", "output", "use <file> as output. Default is to use standard output.", "<file>")
        .optflag("s", "skip", "when decoding, skip newlines. Default is to accept only well-formed input.")
        .optopt("w", "wrap", "when encoding, add newlines every <cols> characters and at the end. Default is to produce well-formed output.", "<cols>")
        ;

    // Deal with --help.
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        let brief = format!("Usage: {} [<options>]", program);
        print!("{}", opts.usage(&brief));
        println!("\nExamples:");
        println!("    {}", program);
        println!("    {} -d", program);
        println!("    {} -b32 -d -s", program);
        println!("    {} -b=0123456789abcdef -w76", program);
        return Ok(());
    }

    // Parse options.
    let matches = try!(opts.parse(&args).map_err(Error::ParseOpts));
    check!(matches.free.len() == 0, Error::ExtraArgs);

    // Deal with --base.
    let name = matches.opt_str("b").unwrap_or("64".into());
    let base = try!(base::lookup(name));

    // Deal with --input.
    let mut input: Box<ReadShift>;
    if let Some(file) = matches.opt_str("i") {
        input = Box::new(try!(File::open(&file).map_err(|e| Error::Open(file, e))));
    } else {
        input = Box::new(std::io::stdin());
    };

    // Deal with --output.
    let mut output: Box<Write>;
    if let Some(file) = matches.opt_str("o") {
        output = Box::new(try!(File::create(&file).map_err(|e| Error::Create(file, e))));
    } else {
        // TODO: Change the following lines when Stdout does not go
        // through a LineWriter anymore.
        output = Box::new(unsafe { <File as std::os::unix::io::FromRawFd>::from_raw_fd(1) });
        // output = Box::new(std::io::stdout());
    }
    output = Box::new(std::io::BufWriter::new(output));

    // Deal with --decode, --wrap, and --skip.
    let operation: Operation;
    let size = 8192;
    let imod;
    let omod;
    if matches.opt_present("d") {
        check!(!matches.opt_present("w"), Error::WrapDecode);
        imod = base.encode_len(1);
        omod = base.decode_len(1);
        operation = Box::new(move |i, o| Ok(try!(base.decode_mut(i, o))));
        if matches.opt_present("s") {
            input = Box::new(Skip::new(input));
        }
    } else {
        check!(!matches.opt_present("s"), Error::SkipEncode);
        imod = base.decode_len(1);
        omod = base.encode_len(1);
        operation = Box::new(move |i, o| { base.encode_mut(i, o); Ok(o.len()) });
        let cols = matches.opt_str("w").unwrap_or("0".into());
        let cols = try!(cols.parse().map_err(Error::ParseWrap));
        if cols > 0 {
            output = Box::new(Wrap::new(output, cols));
        }
    }

    // Do the real work.
    repeat(input, output, operation, size, imod, omod)
}
