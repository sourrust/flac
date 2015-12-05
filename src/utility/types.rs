use nom::{IResult, Needed};

use std::ptr;

pub enum ErrorKind {
  EndOfInput,
  Unknown,
}

pub trait StreamProducer {
  fn parse<F, T>(&mut self, f: F) -> Result<T, ErrorKind>
   where F: FnOnce(&[u8]) -> IResult<&[u8], T>;
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
      IResult::Error(_)      => Err(ErrorKind::Unknown),
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
      let length      = self.filled - self.offset;
      let mut mut_ptr = self.data.as_mut_ptr();

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
