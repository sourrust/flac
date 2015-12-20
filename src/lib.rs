//! An implementation of [FLAC](https://xiph.org/flac), free lossless audio
//! codec, written in Rust.

#[macro_use]
extern crate nom;

#[macro_use]
mod utility;
pub mod metadata;
pub mod frame;
pub mod subframe;
pub mod stream;

pub use metadata::{metadata_parser, Metadata};
pub use frame::{frame_parser, Frame};
pub use subframe::{subframe_parser, Subframe};
pub use stream::{stream_parser, Stream};
