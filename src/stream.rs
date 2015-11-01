use metadata;

use metadata::{StreamInfo, metadata_parser};
use frame::{frame_parser, Frame};

pub struct Stream {
  pub info: StreamInfo,
  pub metadata: Vec<metadata::Metadata>,
  pub frames: Vec<Frame>,
}

named!(pub stream <&[u8], Stream>,
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
