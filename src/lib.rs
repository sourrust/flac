#[macro_use]
extern crate nom;

pub mod metadata;
mod frame;
mod utility;

use metadata::metadata_parser;

pub struct Stream {
  pub info: metadata::StreamInfo,
  pub metadata: Vec<metadata::Block>,
  pub frames: Vec<frame::Frame>
}

named!(stream <&[u8], Stream>,
  chain!(
    blocks: metadata_parser ~
    move|| {
      Stream {
        info: blocks.0,
        metadata: blocks.1,
      }
    }
  )
);
