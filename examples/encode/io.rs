use std::io::{self, Read, Write};
use std::ops::Deref;

use data_encoding::decode;

use error::Error;

pub trait ReadShift: Read {
    fn shift(&self, pos: usize) -> usize;
    fn length(&self) -> usize;
}

impl ReadShift for ::std::fs::File {
    fn shift(&self, pos: usize) -> usize { pos }
    fn length(&self) -> usize { 0 }
}

impl ReadShift for ::std::io::Stdin {
    fn shift(&self, pos: usize) -> usize { pos }
    fn length(&self) -> usize { 0 }
}

impl<T: ReadShift + ?Sized> ReadShift for Box<T> {
    fn shift(&self, pos: usize) -> usize { self.deref().shift(pos) }
    fn length(&self) -> usize { self.deref().length() }
}

pub struct Skip<R: ReadShift> {
    inner: R,
    removed: Vec<usize>,
}

impl<R: ReadShift> Skip<R> {
    pub fn new(inner: R) -> Self {
        Skip { inner: inner, removed: vec![] }
    }
}

impl<R: ReadShift> Read for Skip<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = try!(self.inner.read(buf));
        self.removed.clear();
        let mut src = 0;
        while src < len && buf[src] != b'\n' {
            src += 1;
        }
        let mut dst = src;
        while src < len {
            if buf[src] != b'\n' {
                buf[dst] = buf[src];
                dst += 1;
            } else {
                self.removed.push(src);
            }
            src += 1;
        }
        Ok(dst)
    }
}

impl<R: ReadShift> ReadShift for Skip<R> {
    fn shift(&self, mut pos: usize) -> usize {
        for &i in self.removed.iter() {
            if i > pos {
                break;
            }
            pos += 1;
        }
        self.inner.shift(pos)
    }

    fn length(&self) -> usize {
        self.removed.len()
    }
}

pub struct Wrap<W: Write> {
    inner: W,
    pos: usize,
    max: usize,
}

impl<W: Write> Wrap<W> {
    pub fn new(inner: W, max: usize) -> Self {
        Wrap { inner: inner, pos: 0, max: max }
    }
}

impl<W: Write> Write for Wrap<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        let mut pos = 0;
        while pos < len {
            let next = pos + self.max - self.pos;
            if next <= len {
                try!(self.inner.write_all(&buf[pos .. next]));
                try!(self.inner.write_all(&[b'\n']));
                self.pos = 0;
                pos = next;
            } else {
                try!(self.inner.write_all(&buf[pos ..]));
                self.pos += len - pos;
                pos = len;
            }
        }
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.pos != 0 {
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

fn shift<R: ReadShift>(reader: &R, err: decode::Error, delta: usize) -> Error {
    // TODO: Map decode::Error to custom Error to add the unexpected
    // character too.
    Error::Decode(<decode::Error>::map(err, |pos| delta + reader.shift(pos)))
}

pub type Operation = Box<Fn(&[u8], &mut [u8]) -> Result<usize, decode::Error>>;

pub fn repeat<R: ReadShift, W: Write>
    (mut reader: R, mut writer: W, op: Operation, size: usize, imod: usize, omod: usize) ->
    Result<(), Error>
{
    let mut input = vec![0u8; size];
    let mut output = vec![0u8; (size + imod - 1) / imod * omod];
    let mut pos = 0;
    let mut rest = 0;
    loop {
        let ilen = try!(reader.read(&mut input[rest ..]).map_err(Error::Read));
        let next = if ilen == 0 { rest } else { (rest + ilen) / imod * imod };
        let mlen = (next + imod - 1) / imod * omod;
        let olen = try!(op(&input[0 .. next], &mut output[0 .. mlen])
                        .map_err(|e| shift(&reader, e, pos)));
        try!(writer.write_all(&output[0 .. olen]).map_err(Error::Write));
        if ilen == 0 {
            return Ok(());
        } else if mlen != olen {
            let rest = try!(reader.read(&mut input[0 .. 1]).map_err(Error::Read));
            check!(rest == 0, Error::ExtraInput);
        }
        rest = rest + ilen - next;
        for i in 0 .. rest {
            input[i] = input[next + i];
        }
        pos += next + reader.length();
    }
}
