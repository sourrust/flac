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
    metadata: metadata_parser,
    || {
      Stream {
        metadata: metadata,
      }
    }
  )
);
