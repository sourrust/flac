use nom::{Err, IResult, Needed};

use std::io;
use std::io::Read;
use std::ptr;
use std::cmp;

use super::StreamProducer;

#[derive(Debug)]
pub enum ErrorKind {
  IO(io::Error),
  Incomplete(usize),
  Consumed(usize),
  EndOfInput,
  Unknown,
}

pub struct ByteStream<'a> {
  offset: usize,
  bytes: &'a [u8],
}

impl<'a> ByteStream<'a> {
  pub fn new(bytes: &'a [u8]) -> Self {
    ByteStream {
      offset: 0,
      bytes: bytes,
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.bytes.len() - self.offset
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl<'a> StreamProducer for ByteStream<'a> {
  fn parse<F, T>(&mut self, f: F) -> Result<T, ErrorKind>
   where F: FnOnce(&[u8]) -> IResult<&[u8], T> {
    if self.is_empty() {
      return Err(ErrorKind::EndOfInput);
    }

    match f(&self.bytes[self.offset..]) {
      IResult::Done(i, o)    => {
        self.offset += self.len() - i.len();

        Ok(o)
      }
      IResult::Incomplete(n) => {
        let mut needed = self.len();

        if let Needed::Size(size) = n {
          needed = size;
        }

        Err(ErrorKind::Incomplete(needed))
      }
      IResult::Error(e)      => {
        if let Err::Position(_, i) = e {
          self.offset += self.len() - i.len();
        }

        Err(ErrorKind::Unknown)
      }
    }
  }
}

pub struct Buffer {
  data: Vec<u8>,
  filled: usize,
  offset: usize,
}

impl Buffer {
  pub fn new() -> Self {
    Self::with_capacity(1024)
  }

  pub fn with_capacity(capacity: usize) -> Self {
    let mut buffer = Vec::with_capacity(capacity);

    unsafe {
      buffer.set_len(capacity);
    }

    Buffer {
      data: buffer,
      filled: 0,
      offset: 0,
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.filled - self.offset
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  #[inline]
  pub fn capacity(&self) -> usize {
    self.data.len()
  }

  pub fn as_slice(&self) -> &[u8] {
    &self.data[self.offset..self.filled]
  }

  pub fn fill<R: Read>(&mut self, reader: &mut R) -> io::Result<usize> {
    reader.read(&mut self.data[self.filled..]).map(|consumed| {
      self.filled += consumed;

      consumed
    })
  }

  pub fn resize(&mut self, size: usize) {
    if size > self.data.capacity() {
      self.data.reserve(size);

      let capacity = self.data.capacity();

      unsafe {
        self.data.set_len(capacity);
      }
    }

    if self.data.len() - self.filled < size  {
      let length  = self.filled - self.offset;
      let mut_ptr = self.data.as_mut_ptr();

      unsafe {
        let offset_ptr  = self.data.as_ptr().offset(self.offset as isize);

        ptr::copy(offset_ptr, mut_ptr, length);
      }

      self.filled -= self.offset;
      self.offset  = 0;
    }
  }

  pub fn consume(&mut self, consumed: usize) {
    self.offset += consumed;
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
  Incomplete,
  EndOfInput,
}

fn fill<R: Read>(buffer: &mut Buffer, reader: &mut R, needed: usize)
                 -> io::Result<usize> {
  let mut read = 0;

  if buffer.len() < needed {
    buffer.resize(needed);

    while buffer.len() < needed {
      let size_read = try!(buffer.fill(reader));

      if size_read > 0 {
        read += size_read;
      } else {
        break;
      }
    }
  }

  Ok(read)
}

pub struct ReadStream<R: Read> {
  reader: R,
  buffer: Buffer,
  needed: usize,
  state: ParserState,
}

impl<R> ReadStream<R> where R: Read {
  pub fn new(reader: R) -> Self {
    ReadStream {
      reader: reader,
      buffer: Buffer::new(),
      needed: 0,
      state: ParserState::Incomplete,
    }
  }

  fn fill(&mut self) -> io::Result<usize> {
    let needed = cmp::max(1, self.needed);

    fill(&mut self.buffer, &mut self.reader, needed).map(|consumed| {
      if self.buffer.len() < needed {
        self.state = ParserState::EndOfInput;
      }

      consumed
    })
  }
}

fn from_iresult<T>(buffer: &Buffer, result: IResult<&[u8], T>)
                   -> Result<(usize, T), ErrorKind> {
  match result {
    IResult::Done(i, o)    => Ok((buffer.len() - i.len(), o)),
    IResult::Incomplete(n) => {
      let mut needed = buffer.capacity();

      if let Needed::Size(size) = n {
        needed = size;
      }

      Err(ErrorKind::Incomplete(needed))
    }
    IResult::Error(e)      => {
      if let Err::Position(_, i) = e {
        Err(ErrorKind::Consumed(buffer.len() - i.len()))
      } else {
        Err(ErrorKind::Unknown)
      }
    }
  }
}

impl<R> StreamProducer for ReadStream<R> where R: Read {
  fn parse<F, T>(&mut self, f: F) -> Result<T, ErrorKind>
   where F: FnOnce(&[u8]) -> IResult<&[u8], T> {
    if self.state != ParserState::EndOfInput {
      try!(self.fill().map_err(ErrorKind::IO));
    }

    let mut buffer = &mut self.buffer;

    if buffer.is_empty() {
      self.state = ParserState::EndOfInput;

      return Err(ErrorKind::EndOfInput);
    }

    let result = {
      let iresult = f(buffer.as_slice());

      from_iresult(&buffer, iresult)
    };

    match result {
      Ok((consumed, o)) => {
        buffer.consume(consumed);

        Ok(o)
      }
      Err(kind)         => {
        match kind {
          ErrorKind::Incomplete(needed) => self.needed = needed,
          ErrorKind::Consumed(consumed) => buffer.consume(consumed),
          _                             => (),
        }

        Err(kind)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use utility::StreamProducer;
  use nom::be_u32;

  #[test]
  fn test_buffer() {
    let mut buffer = Buffer::new();
    let bytes      = b"Hello World";
    let mut reader = &bytes[..];

    assert!(buffer.is_empty());
    assert_eq!(buffer.capacity(), 1024);

    let bytes_read = buffer.fill(&mut reader).unwrap_or(0);
    let bytes_len  = bytes.len();

    assert_eq!(bytes_read, bytes_len);
    assert_eq!(buffer.len(), bytes_len);
    assert_eq!(buffer.as_slice(), bytes);

    buffer.resize(512);
    assert_eq!(buffer.capacity(), 1024);
  }

  #[test]
  fn test_read_stream() {
    let bytes      = b"Hello World";
    let mut stream = ReadStream::new(&bytes[..]);

    let result = stream.parse(be_u32).unwrap_or(0);

    assert_eq!(result, 1214606444)
  }
}
