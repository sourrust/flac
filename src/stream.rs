use nom::{
  Consumer, ConsumerState,
  ErrorKind,
  HexDisplay,
  Producer, FileProducer,
  Input, IResult,
  Move, Needed,
};

use metadata;

use metadata::{Metadata, StreamInfo, metadata_parser};
use frame::{frame_parser, Frame};
use utility::resize_producer;

use std::io;
use std::io::{Error, Result};

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
  consumer_state: ConsumerState<(), ErrorKind, Move>,
}

named!(pub stream_parser <&[u8], Stream>,
  chain!(
    blocks: metadata_parser ~
    frames: many1!(apply!(frame_parser, &blocks.0)),
    move|| {
      let consumed = Move::Consume(0);

      Stream {
        info: blocks.0,
        metadata: blocks.1,
        frames: frames,
        state: ParserState::Marker,
        consumer_state: ConsumerState::Continue(consumed),
      }
    }
  )
);

impl Stream {
  pub fn new() -> Stream {
    let consumed = Move::Consume(0);

    Stream {
      info: StreamInfo::new(),
      metadata: Vec::new(),
      frames: Vec::new(),
      state: ParserState::Marker,
      consumer_state: ConsumerState::Continue(consumed),
    }
  }

  pub fn info(&self) -> StreamInfo {
    self.info
  }

  pub fn from_file(filename: &str) -> Result<Stream> {
    FileProducer::new(filename, 1024).and_then(|mut producer| {
      let consumed        = Move::Consume(0);
      let mut buffer_size = 1024;
      let mut is_error    = false;
      let mut stream      = Stream {
        info: StreamInfo::new(),
        metadata: Vec::new(),
        frames: Vec::new(),
        state: ParserState::Marker,
        consumer_state: ConsumerState::Continue(consumed),
      };

      loop {
        match *producer.apply(&mut stream) {
          ConsumerState::Done(_, _)      => break,
          ConsumerState::Continue(await) => {
            let result = resize_producer(&mut producer, &await, buffer_size);

            if let Some(size) = result {
              buffer_size = size;
            }

            continue;
          }
          ConsumerState::Error(_)        => {
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

        Err(Error::new(io::ErrorKind::InvalidData, error_str))
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

  fn handle_metadata(&mut self, input: &[u8]) {
    match metadata::block(input) {
      IResult::Done(i, block) => {
        let offset   = input.offset(i);
        let consumed = Move::Consume(offset);
        let is_last  = block.is_last;

        if let metadata::Data::StreamInfo(info) = block.data {
          self.info = info;
        } else {
          self.metadata.push(block);
        }

        if is_last {
          self.state = ParserState::Frame;
        }

        self.consumer_state = ConsumerState::Continue(consumed);
      }
      IResult::Error(_)       => {
        let kind = ErrorKind::Custom(1);

        self.consumer_state = ConsumerState::Error(kind);
      }
      IResult::Incomplete(s)  => {
        let size = if let Needed::Size(length) = s {
          length
        } else {
          1024
        };
        let needed = Move::Await(Needed::Size(size));

        self.consumer_state = ConsumerState::Continue(needed);
      }
    }
  }

  fn handle_frame(&mut self, input: &[u8]) {
    match frame_parser(input, &self.info) {
      IResult::Done(i, frame) => {
        let offset   = input.offset(i);
        let consumed = Move::Consume(offset);

        self.frames.push(frame);

        self.consumer_state = ConsumerState::Continue(consumed);
      }
      IResult::Error(_)       => {
        let kind = ErrorKind::Custom(2);

        self.consumer_state = ConsumerState::Error(kind);
      }
      IResult::Incomplete(s)  => {
        let size = if let Needed::Size(length) = s {
          length
        } else {
          self.info.max_frame_size as usize
        };
        let needed = Move::Await(Needed::Size(size));

        self.consumer_state = ConsumerState::Continue(needed);
      }
    }
  }
}

impl<'a> Consumer<&'a [u8], (), ErrorKind, Move> for Stream {
  fn state(&self) -> &ConsumerState<(), ErrorKind, Move> {
    &self.consumer_state
  }

  fn handle(&mut self, input: Input<&'a [u8]>)
            -> &ConsumerState<(), ErrorKind, Move> {
    match input {
      Input::Element(i) | Input::Eof(Some(i)) => {
        match self.state {
          ParserState::Marker   => self.handle_marker(i),
          ParserState::Metadata => self.handle_metadata(i),
          ParserState::Frame    => self.handle_frame(i),
        }
      }
      Input::Empty | Input::Eof(None)         => {
        self.consumer_state = match self.state {
          ParserState::Marker   => ConsumerState::Error(ErrorKind::Custom(0)),
          ParserState::Metadata => ConsumerState::Error(ErrorKind::Custom(1)),
          ParserState::Frame    => ConsumerState::Done(Move::Consume(0), ()),
        };
      }
    }

    &self.consumer_state
  }
}
