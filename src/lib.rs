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
