#![warn(unused_results)]

// TODO: Error messages can be improved (like file position)

extern crate getopts;
extern crate data_encoding;

use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Deref;

use getopts::Options;

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

fn main() {
    let (program, args): (_, Vec<_>) = {
        let mut i = std::env::args();
        (i.next().unwrap_or("encode".into()), i.collect())
    };

    if let Err(top) = wrapped_main(&program, args) {
        let mut cur: Option<&Error> = Some(top.deref());
        while let Some(err) = cur {
            let _ = writeln!(&mut io::stderr(), "{}: {}", &program, err);
            cur = err.cause();
        }
        std::process::exit(1)
    }
}

fn wrapped_main(program: &str, args: Vec<String>) -> Result<(), Box<Error + Send + Sync>> {
    // Define options.
    let mut opts = Options::new();
    let _ = opts
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
        println!("    {} -b64 -w76", program);
        println!("    {} -b=0123456789abcdef -d -s", program);
        return Ok(());
    }

    // Parse options.
    let matches = try!(opts.parse(&args));
    if matches.free.len() != 0 {
        return Err("Extra arguments given.".into());
    }

    // Deal with --base.
    let name = matches.opt_str("b").unwrap_or("64".into());
    let base = try!(lookup(name));

    // Deal with --input.
    let mut input: Box<Read>;
    if let Some(file) = matches.opt_str("i") {
        input = Box::new(try!(File::open(file)));
    } else {
        input = Box::new(io::stdin());
    };

    // Deal with --output.
    let mut output: Box<Write>;
    if let Some(file) = matches.opt_str("o") {
        output = Box::new(try!(File::create(file)));
    } else {
        // output = Box::new(unsafe { <File as std::os::unix::io::FromRawFd>::from_raw_fd(1) });
        output = Box::new(io::stdout());
    }
    output = Box::new(io::BufWriter::new(output));

    // Deal with --decode, --wrap, and --skip.
    let operation: Operation;
    let size = 8192;
    let imod;
    let omod;
    if matches.opt_present("d") {
        if matches.opt_present("w") {
            return Err("Option --wrap is incompatible with --decode.".into());
        }
        imod = base.encode_len(1);
        omod = base.decode_len(1);
        operation = Box::new(move |i, o| Ok(try!(base.decode_mut(i, o))));
        if matches.opt_present("s") {
            input = Box::new(Skip::new(input));
        }
    } else {
        if matches.opt_present("s") {
            panic!("Option --skip is only compatible with --decode.");
        }
        imod = base.decode_len(1);
        omod = base.encode_len(1);
        operation = Box::new(move |i, o| { base.encode_mut(i, o); Ok(o.len()) });
        let cols = try!(matches.opt_str("w").unwrap_or("0".into()).parse());
        if cols > 0 {
            output = Box::new(Wrap::new(output, cols));
        }
    }

    // Do the real work.
    repeat(input, output, operation, size, imod, omod)
}

trait Base {
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

fn lookup(b: String) -> Result<Box<Base>, Box<Error + Send + Sync>> {
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
        _ => Ok(Box::new(try!(build(b)))),
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

fn build(b: String) -> Result<Custom, Box<Error + Send + Sync>> {
    let mut i = b.as_bytes().iter().cloned();
    let pad = try!(i.next().ok_or("No padding for --base."));
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
    if sym.len() != 1 << bit as usize {
        return Err("Invalid base: Not a power of two.".into());
    }
    let mut val = vec![128u8; 256];
    for (i, &s) in sym.iter().enumerate() {
        val[s as usize] = i as u8;
    }
    let base = Custom { val: val, sym: sym, bit: bit, pad: pad };
    match base::valid(&base) {
        Ok(()) => Ok(base),
        Err(e) => Err(format!("Invalid base: {:?}", e).into()),
    }
}

struct Skip<R: Read> { inner: R }

impl<R: Read> Skip<R> {
    fn new(inner: R) -> Self {
        Skip { inner: inner }
    }
}

impl<R: Read> Read for Skip<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = try!(self.inner.read(buf));
        let mut src = 0;
        while src < len && buf[src] != b'\n' {
            src += 1;
        }
        let mut dst = src;
        while src < len {
            if buf[src] != b'\n' {
                buf[dst] = buf[src];
                dst += 1;
            }
            src += 1;
        }
        Ok(dst)
    }
}

struct Wrap<W: Write> { inner: W, rem: usize, modulo: usize }

impl<W: Write> Wrap<W> {
    fn new(inner: W, modulo: usize) -> Self {
        Wrap { inner: inner, rem: 0, modulo: modulo }
    }
}

impl<W: Write> Write for Wrap<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        let mut pos = 0;
        while pos < len {
            let next = pos + self.modulo - self.rem;
            if next <= len {
                try!(self.inner.write_all(&buf[pos .. next]));
                try!(self.inner.write_all(&[b'\n']));
                self.rem = 0;
                pos = next;
            } else {
                try!(self.inner.write_all(&buf[pos ..]));
                self.rem += len - pos;
                pos = len;
            }
        }
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.rem != 0 {
            try!(self.inner.write_all(&[b'\n']));
        }
        self.inner.flush()
    }
}

impl<W: Write> Drop for Wrap<W> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}

type Operation = Box<Fn(&[u8], &mut [u8]) -> Result<usize, Box<Error + Send + Sync>>>;

fn repeat<R: Read, W: Write>
    (mut reader: R, mut writer: W, fun: Operation, size: usize, imod: usize, omod: usize)
     -> Result<(), Box<Error + Send + Sync>>
{
    let mut input = vec![0u8; size];
    let mut output = vec![0u8; (size + imod - 1) / imod * omod];
    let mut rest = 0;
    loop {
        let ilen = try!(reader.read(&mut input[rest ..]));
        let next = if ilen == 0 { rest } else { (rest + ilen) / imod * imod };
        let mlen = (next + imod - 1) / imod * omod;
        let olen = try!(fun(&input[0 .. next], &mut output[0 .. mlen]));
        try!(writer.write_all(&output[0 .. olen]));
        if ilen == 0 {
            return Ok(());
        } else if mlen != olen {
            if try!(reader.read(&mut input[0 .. 1])) == 0 {
                return Ok(());
            } else {
                return Err("Decoding ended before end-of-file.".into());
            }
        }
        rest = rest + ilen - next;
        for i in 0 .. rest {
            input[i] = input[next + i];
        }
    }
}
