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
      }
    }
  )
);
