use nom::{ConsumerState, ErrorKind, HexDisplay, IResult, Move};


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
}
