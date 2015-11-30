use nom::{ConsumerState, Move};


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
