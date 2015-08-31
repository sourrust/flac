#[macro_use]
extern crate nom;

pub mod metadata;
mod frame;
mod utility;

use metadata::metadata_parser;
use frame::frame_parser;

pub struct Stream {
  pub info: metadata::StreamInfo,
  pub metadata: Vec<metadata::Block>,
  pub frames: Vec<frame::Frame>
}

named!(stream <&[u8], Stream>,
  chain!(
    blocks: metadata_parser ~
    frames: many1!(apply!(frame_parser, blocks.0.channels)),
    move|| {
      Stream {
        info: blocks.0,
        metadata: blocks.1,
        frames: frames,
      }
    }
  )
);
