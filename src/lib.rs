#[macro_use]
extern crate nom;

pub mod metadata;
mod utility;

use metadata::metadata;

pub struct Stream<'a> {
  pub meta_data: Vec<metadata::Block<'a>>,
  //frames: Vec<u32>
}

named!(stream <&[u8], Stream>,
  chain!(
    blocks: metadata,
    || {
      Stream {
        meta_data: blocks,
      }
    }
  )
);
