use nom;
use nom::{Err, IResult};

use metadata;

use metadata::{Metadata, StreamInfo, metadata_parser};
use frame::{frame_parser, Frame};
use utility::{ErrorKind, ByteStream, ReadStream, StreamProducer};

use std::io;
use std::fs::File;

enum ParserState {
  Marker,
  Metadata,
  Frame,
}

pub struct Stream {
  info: StreamInfo,
  pub metadata: Vec<Metadata>,
  pub frames: Vec<Frame>,
  state: ParserState,
  output: Vec<i32>,
  frame_index: usize,
}

named!(pub stream_parser <&[u8], Stream>,
  chain!(
    blocks: metadata_parser ~
    frames: many1!(apply!(frame_parser, &blocks.0)),
    move|| {
      Stream {
        info: blocks.0,
        metadata: blocks.1,
        frames: frames,
        state: ParserState::Marker,
        output: Vec::new(),
        frame_index: 0,
      }
    }
  )
);

impl Stream {
  pub fn new() -> Stream {
    Stream {
      info: StreamInfo::new(),
      metadata: Vec::new(),
      frames: Vec::new(),
      state: ParserState::Marker,
      output: Vec::new(),
      frame_index: 0,
    }
  }

  pub fn info(&self) -> StreamInfo {
    self.info
  }

  pub fn from_file(filename: &str) -> io::Result<Stream> {
    File::open(filename).and_then(|file| {
      let mut producer = ReadStream::new(file);
      let error_str    = format!("parser: couldn't parse the given file {}",
                                 filename);

      Stream::from_stream_producer(&mut producer, &error_str)
    })
  }

  pub fn from_buffer(buffer: &[u8]) -> io::Result<Stream> {
    let mut producer = ByteStream::new(buffer);
    let error_str    = "parser: couldn't parse the buffer";

    Stream::from_stream_producer(&mut producer, error_str)
  }

  fn from_stream_producer<P>(producer: &mut P, error_str: &str)
                             -> io::Result<Stream>
   where P: StreamProducer {
    let mut is_error = false;
    let mut stream   = Stream {
      info: StreamInfo::new(),
      metadata: Vec::new(),
      frames: Vec::new(),
      state: ParserState::Marker,
      output: Vec::new(),
      frame_index: 0,
    };

    loop {
      match stream.handle(producer) {
        Ok(_)                         => break,
        Err(ErrorKind::EndOfInput)    => break,
        Err(ErrorKind::Consumed(_))   => continue,
        Err(ErrorKind::Incomplete(_)) => continue,
        Err(_)                        => {
          is_error = true;

          break;
        }
      }
    }

    if !is_error {
      let channels    = stream.info.channels as usize;
      let block_size  = stream.info.max_block_size as usize;
      let output_size = block_size * channels;

      stream.output.reserve_exact(output_size);

      unsafe { stream.output.set_len(output_size) }

      Ok(stream)
    } else {
      Err(io::Error::new(io::ErrorKind::InvalidData, error_str))
    }
  }

  fn handle_marker<'a>(&mut self, input: &'a [u8]) -> IResult<&'a [u8], ()> {
    let kind = nom::ErrorKind::Custom(0);

    match tag!(input, "fLaC") {
      IResult::Done(i, _)    => {
        self.state = ParserState::Metadata;

        IResult::Error(Err::Position(kind, i))
      }
      IResult::Error(_)      => IResult::Error(Err::Code(kind)),
      IResult::Incomplete(n) => IResult::Incomplete(n),
    }
  }

  fn handle_metadata<'a>(&mut self, input: &'a [u8])
                         -> IResult<&'a [u8], ()> {
    let kind = nom::ErrorKind::Custom(1);

    match metadata::block(input) {
      IResult::Done(i, block) => {
        let is_last = block.is_last;

        if let metadata::Data::StreamInfo(info) = block.data {
          self.info = info;
        } else {
          self.metadata.push(block);
        }

        if is_last {
          self.state = ParserState::Frame;
        }

        IResult::Error(Err::Position(kind, i))
      }
      IResult::Error(_)      => IResult::Error(Err::Code(kind)),
      IResult::Incomplete(n) => IResult::Incomplete(n),
    }
  }

  fn handle_frame<'a>(&mut self, input: &'a [u8]) -> IResult<&'a [u8], ()> {
    let kind = nom::ErrorKind::Custom(2);

    match frame_parser(input, &self.info) {
      IResult::Done(i, frame) => {
        self.frames.push(frame);

        IResult::Error(Err::Position(kind, i))
      }
      IResult::Error(_)      => IResult::Error(Err::Code(kind)),
      IResult::Incomplete(n) => IResult::Incomplete(n),
    }
  }

  pub fn handle<S: StreamProducer>(&mut self, stream: &mut S)
                                   -> Result<(), ErrorKind> {
    stream.parse(|input| {
      match self.state {
        ParserState::Marker   => self.handle_marker(input),
        ParserState::Metadata => self.handle_metadata(input),
        ParserState::Frame    => self.handle_frame(input),
      }
    })
  }
}
