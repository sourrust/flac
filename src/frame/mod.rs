mod types;
mod parser;
mod decoder;

pub use self::types::{
  MAX_CHANNELS,
  ChannelAssignment, NumberType,
  Frame,
  Header, Footer,
};

pub use self::parser::frame_parser;
pub use self::decoder::decode;
