/// Maximum number of channels supported in the FLAC format.
pub const MAX_CHANNELS: usize = 8;

/// Audio frame that contains one sample for each channel.
pub struct Frame {
  /// Information regarding the current audio frame.
  pub header: Header,
  /// CRC-16 of all frame bytes before this footer.
  pub footer: Footer,
}

/// Channel assignment order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelAssignment {
  /// Independent channels, from one up to eight.
  Independent,
  /// Left and side stereo.
  LeftSide,
  /// Right and side stereo.
  RightSide,
  /// Middle and side stereo.
  MiddleSide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberType {
  Frame(u32),
  Sample(u64),
}

/// Information regarding the current audio frame.
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

/// End of the audio frame.
///
/// Contains a value that represents the CRC-16 of everything inside the
/// frame before the footer.
#[derive(Debug, PartialEq, Eq)]
pub struct Footer(pub u16);
