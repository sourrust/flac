//! An implementation of [FLAC](https://xiph.org/flac), free lossless audio
//! codec, written in Rust.

#[macro_use]
extern crate nom;

#[macro_use]
mod utility;
pub mod metadata;
pub mod frame;

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
