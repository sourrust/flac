#[macro_use]
extern crate nom;

pub mod metadata;
mod utility;

use metadata::metadata;

pub struct Stream<'a> {
  pub metadata: Vec<metadata::Block<'a>>,
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
