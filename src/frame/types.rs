/// Maximum number of channels supported in the FLAC format.
pub const MAX_CHANNELS: usize = 8;

pub struct Frame {
  pub header: Header,
  pub footer: Footer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelAssignment {
  Independent = 0,
  LeftSide    = 1,
  RightSide   = 2,
  MiddleSide  = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberType {
  Frame(u32),
  Sample(u64),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
  pub block_size: u32,
  pub sample_rate: u32,
  pub channels: u8,
  pub channel_assignment: ChannelAssignment,
  pub bits_per_sample: usize,
  pub number: NumberType,
  pub crc: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Footer(pub u16);
