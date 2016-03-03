use nom::{self, IResult, Needed};

use std::io::{self, Read};
use std::ptr;
use std::cmp;

use super::StreamProducer;

/// Represent the different kinds of errors.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
  /// Error from I/O.
  IO(io::ErrorKind),
  /// A parser stopped midway and need more bytes to consume.
  Incomplete(usize),
  /// A parser has completes and there is still more bytes to consume.
  Continue,
  /// A parser has completes and there is no more bytes to consume.
  EndOfInput,
  /// A non-specified error.
  Unknown,
  // Parser Error
  /// Failed parsing the "fLaC" header token.
  HeaderParser,
  /// Failed parsing a metadata header.
  MetadataHeaderParser,
  /// Failed parsing the metadata block `StreamInfo`.
  StreamInfoParser,
  /// Failed parsing the metadata block `Padding`.
  PaddingParser,
  /// Failed parsing the metadata block `Application`.
  ApplicationParser,
  /// Failed parsing the metadata block `SeekTable`.
  SeekTableParser,
  /// Failed parsing the metadata block `VorbisComment`.
  VorbisCommentParser,
  /// Failed parsing the metadata block `CueSheet`.
  CueSheetParser,
  /// Failed parsing the metadata block `Picture`.
  PictureParser,
  /// Failed parsing the metadata block `Unknown`.
  UnknownParser,
  BlockingStrategyParser,
  BlockingSampleParser,
  ChannelBitsParser,
  UTF8HeaderParser,
  UTF8BodyParser,
  CRC8Parser,
  FrameFooterParser,
  // Invalid Error
  /// A block type, base on the number, that is outside the range (0-126).
  InvalidBlockType,
  InvalidSyncCode,
  InvalidBlockSample,
  InvalidChannelBits,
  InvalidUTF8,
  InvalidCRC8,
  InvalidCRC16,
  // Not Found
  NotFound,
}

/// Structure that hold a slice of bytes.
pub struct ByteStream<'a> {
  offset: usize,
  bytes: &'a [u8],
}

impl<'a> ByteStream<'a> {
  /// Construct a `ByteStream` based on the passed in byte slice.
  pub fn new(bytes: &'a [u8]) -> Self {
    ByteStream {
      offset: 0,
      bytes: bytes,
    }
  }

  /// Return the number of bytes that haven't been consumed yet.
  #[inline]
  pub fn len(&self) -> usize {
    self.bytes.len() - self.offset
  }

  /// Return true if the stream contains no more bytes.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl<'a> StreamProducer for ByteStream<'a> {
  fn parse<F, T>(&mut self, f: F) -> Result<T, ErrorKind>
   where F: FnOnce(&[u8]) -> IResult<&[u8], T, ErrorKind> {
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
        match e {
          nom::Err::Code(k)               |
          nom::Err::Node(k, _)            |
          nom::Err::Position(k, _)        |
          nom::Err::NodePosition(k, _, _) => {
            if let nom::ErrorKind::Custom(kind) = k {
              Err(kind)
            } else {
              Err(ErrorKind::Unknown)
            }
          }
        }
      },
    }
  }
}

// Growable buffer of bytes.
//
// Mainly used to the `ReadStream` structure but can be used seperately for
// manually filling with some `Read` source.
pub struct Buffer {
  data: Vec<u8>,
  filled: usize,
  offset: usize,
}

impl Buffer {
  // Default constructor for `Buffer`
  pub fn new() -> Self {
    Self::with_capacity(1024)
  }

  // Explicitly set the buffer capacity.
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

  // Return the number of read bytes that haven't been consumed yet.
  #[inline]
  pub fn len(&self) -> usize {
    self.filled - self.offset
  }

  // Return true if buffer contains no more bytes.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  // The set length of the unlining buffer.
  #[inline]
  pub fn capacity(&self) -> usize {
    self.data.len()
  }

  // Return a reference to the slice of unread bytes.
  pub fn as_slice(&self) -> &[u8] {
    &self.data[self.offset..self.filled]
  }

  // Fill the buffer with bytes from a `Read` source.
  pub fn fill<R: Read>(&mut self, reader: &mut R) -> io::Result<usize> {
    reader.read(&mut self.data[self.filled..]).map(|consumed| {
      self.filled += consumed;

      consumed
    })
  }

  // Resize the current buffer
  //
  // This will only allocate data when the size requests is larger than the
  // current capacity of the buffer, otherwise it moves the currently filled
  // data to the beginning of the buffer.
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

  // Move the offset by the amount of consumed bytes.
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

/// Structure that hold a reader for a source of bytes.
pub struct ReadStream<R: Read> {
  reader: R,
  buffer: Buffer,
  needed: usize,
  state: ParserState,
}

impl<R> ReadStream<R> where R: Read {
  /// Constructor for `ReadStream` based on a `Read` source.
  pub fn new(reader: R) -> Self {
    ReadStream {
      reader: reader,
      buffer: Buffer::new(),
      needed: 0,
      state: ParserState::Incomplete,
    }
  }

  // Fill the stream with bytes from a `Read` source.
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

fn from_iresult<T>(buffer: &Buffer, result: IResult<&[u8], T, ErrorKind>)
                   -> Result<(usize, T), ErrorKind> {
  match result {
    IResult::Done(i, o)    => Ok((buffer.len() - i.len(), o)),
    IResult::Incomplete(n) => {
      let mut needed = buffer.capacity() + 1024;

      if let Needed::Size(size) = n {
        needed = size;
      }

      Err(ErrorKind::Incomplete(needed))
    }
    IResult::Error(e)      => {
      match e {
        nom::Err::Code(k)               |
        nom::Err::Node(k, _)            |
        nom::Err::Position(k, _)        |
        nom::Err::NodePosition(k, _, _) => {
          if let nom::ErrorKind::Custom(kind) = k {
            Err(kind)
          } else {
            Err(ErrorKind::Unknown)
          }
        }
      }
    },
  }
}

impl<R> StreamProducer for ReadStream<R> where R: Read {
  fn parse<F, T>(&mut self, f: F) -> Result<T, ErrorKind>
   where F: FnOnce(&[u8]) -> IResult<&[u8], T, ErrorKind> {
    if self.state != ParserState::EndOfInput {
      try!(self.fill().map_err(|e| ErrorKind::IO(e.kind())));
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
        if let ErrorKind::Incomplete(needed) = kind {
          self.needed = needed;

          Err(ErrorKind::Continue)
        } else {
          Err(kind)
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use utility::StreamProducer;
  use nom::{self, IResult};

  #[inline]
  fn be_u32(input: &[u8]) -> IResult<&[u8], u32, ErrorKind> {
    nom::be_u32(input).map_err(to_custom_error!(Unknown))
  }

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
  fn test_byte_stream() {
    let bytes      = b"Hello World";
    let mut stream = ByteStream::new(bytes);

    assert_eq!(stream.len(), bytes.len());

    let result = stream.parse(be_u32).unwrap_or(0);

    assert_eq!(result, 1214606444);
    assert_eq!(stream.len(), 7);
  }

  #[test]
  fn test_read_stream() {
    let bytes      = b"Hello World";
    let mut stream = ReadStream::new(&bytes[..]);

    let result = stream.parse(be_u32).unwrap_or(0);

    assert_eq!(result, 1214606444)
  }
}
