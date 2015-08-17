#[macro_use]
extern crate nom;

pub mod metadata;
mod utility;

use metadata::metadata;

pub struct Stream {
  pub metadata: Vec<metadata::Block>,
  //frames: Vec<u32>
}

named!(stream <&[u8], Stream>,
  chain!(
    blocks: metadata,
    || {
      Stream {
        metadata: blocks,
      }
    }
  )
);
