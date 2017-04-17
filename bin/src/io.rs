use base::Base;
use data_encoding::DecodeError;
use error::Error;
use std::io::{Read, Write};
use std::ops::Deref;

pub trait ReadDelta: Read {
    fn delta(&self, pos: usize) -> usize;
    fn length(&self) -> usize;
}

impl ReadDelta for ::std::fs::File {
    fn delta(&self, _: usize) -> usize { 0 }
    fn length(&self) -> usize { 0 }
}

impl ReadDelta for ::std::io::Stdin {
    fn delta(&self, _: usize) -> usize { 0 }
    fn length(&self) -> usize { 0 }
}

impl<T: ReadDelta + ?Sized> ReadDelta for Box<T> {
    fn delta(&self, pos: usize) -> usize { self.deref().delta(pos) }
    fn length(&self) -> usize { self.deref().length() }
}

pub struct Skip<R: ReadDelta> {
    inner: R,
    removed: Vec<usize>,
}

impl<R: ReadDelta> Skip<R> {
    pub fn new(inner: R) -> Self {
        Skip { inner: inner, removed: vec![] }
    }
}

impl<R: ReadDelta> Read for Skip<R> {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        self.removed.clear();
        let mut dst = 0;
        while dst < buf.len() {
            let len = self.inner.read(&mut buf[dst ..])?;
            if len == 0 { break; }
            let delta = self.removed.len();
            let end = dst + len;
            while dst < end && buf[dst] != b'\n' { dst += 1; }
            let mut src = dst;
            while src < end {
                if buf[src] != b'\n' {
                    buf[dst] = buf[src];
                    dst += 1;
                } else {
                    self.removed.push(src + delta);
                }
                src += 1;
            }
        }
        Ok(dst)
    }
}

impl<R: ReadDelta> ReadDelta for Skip<R> {
    fn delta(&self, pos: usize) -> usize {
        let mut delta = 0;
        for &i in self.removed.iter() {
            if i > pos + delta { break; }
            delta += 1;
        }
        delta + self.inner.delta(pos + delta)
    }

    fn length(&self) -> usize { self.removed.len() }
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
    fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize> {
        let len = buf.len();
        let mut pos = 0;
        while pos < len {
            let next = pos + self.max - self.pos;
            if next <= len {
                self.inner.write_all(&buf[pos .. next])?;
                self.inner.write_all(&[b'\n'])?;
                self.pos = 0;
                pos = next;
            } else {
                self.inner.write_all(&buf[pos ..])?;
                self.pos += len - pos;
                pos = len;
            }
        }
        Ok(len)
    }

    fn flush(&mut self) -> ::std::io::Result<()> {
        if self.pos != 0 { self.inner.write_all(&[b'\n'])?; }
        self.inner.flush()
    }
}

impl<W: Write> Drop for Wrap<W> {
    fn drop(&mut self) { self.flush().unwrap() }
}

fn floor(x: usize, d: usize) -> usize { x / d * d }
fn ceil(x: usize, d: usize) -> usize { floor(x + d - 1, d) }

#[test]
fn floor_ceil() {
    assert_eq!(floor(10, 5), 10);
    assert_eq!(floor(13, 5), 10);
    assert_eq!(floor(15, 5), 15);
    assert_eq!(ceil(10, 5), 10);
    assert_eq!(ceil(13, 5), 15);
    assert_eq!(ceil(15, 5), 15);
}

pub fn encode<R, W>(base: Base, mut reader: R, mut writer: W, size: usize)
                    -> Result<(), Error> where R: ReadDelta, W: Write {
    let block = base.encode_block();
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
        if ilen == 0 { return Ok(()); }
        rest = rest + ilen - next;
        for i in 0 .. rest {
            input[i] = input[next + i];
        }
    }
}

fn shift<R: ReadDelta>(reader: &R, mut err: DecodeError, delta: usize)
                       -> Error {
    err.position += delta + reader.delta(err.position);
    Error::Decode(err)
}

pub fn decode<R, W>(base: Base, mut reader: R, mut writer: W, size: usize)
                    -> Result<(), Error> where R: ReadDelta, W: Write {
    let block = base.decode_block();
    assert!(size >= block);
    let mut input = vec![0u8; size];
    let mut output = vec![0u8; base.decode_len(ceil(size, block)).unwrap()];
    let mut pos = 0;
    let mut rest = 0;
    loop {
        let ilen = reader.read(&mut input[rest ..]).map_err(Error::Read)?;
        let next = if ilen == 0 { rest } else { floor(rest + ilen, block) };
        let mlen = base.decode_len(next).map_err(|e| shift(&reader, e, pos))?;
        let olen = base.decode_mut(&input[0 .. next], &mut output[0 .. mlen])
            .map_err(|e| shift(&reader, e, pos))?;
        writer.write_all(&output[0 .. olen]).map_err(Error::Write)?;
        if ilen == 0 { return Ok(()); }
        rest = rest + ilen - next;
        for i in 0 .. rest {
            input[i] = input[next + i];
        }
        pos += next + reader.length();
    }
}
