use nom::{
  Consumer, ConsumerState,
  ErrorKind,
  HexDisplay,
  Input, IResult,
  Move, Needed,
};

use metadata;

use metadata::{Metadata, StreamInfo, metadata_parser};
use frame::{frame_parser, Frame};

enum ParserState {
  Marker,
  Metadata,
  Frame,
}

pub struct Stream {
  pub info: StreamInfo,
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
  fn handle_marker(&mut self, input: &[u8]) {
    match tag!(input, "fLaC") {
      IResult::Done(i, _)       => {
        let offset   = input.offset(i);
        let consumed = Move::Consume(offset);

        self.state          = ParserState::Metadata;
        self.consumer_state = ConsumerState::Continue(consumed);
      }
      IResult::Error(_)         => {
        let kind = ErrorKind::Custom(0);

        self.consumer_state = ConsumerState::Error(kind);
      }
      IResult::Incomplete(size) => {
        let needed = Move::Await(size);

        self.consumer_state = ConsumerState::Continue(needed);
      }
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
