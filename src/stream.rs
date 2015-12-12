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
    }
  }

  pub fn info(&self) -> StreamInfo {
    self.info
  }

  pub fn from_file(filename: &str) -> io::Result<Stream> {
    File::open(filename).and_then(|file| {
      let mut reader   = ReadStream::new(file);
      let mut is_error = false;
      let mut stream   = Stream {
        info: StreamInfo::new(),
        metadata: Vec::new(),
        frames: Vec::new(),
        state: ParserState::Marker,
      };

      loop {
        match stream.handle(&mut reader) {
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
        Ok(stream)
      } else {
        let error_str = format!("parser: couldn't parse the given file {}",
                                filename);

        Err(io::Error::new(io::ErrorKind::InvalidData, error_str))
      }
    })
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

    match metadata_parser(input) {
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
