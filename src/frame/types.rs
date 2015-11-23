use subframe::Subframe;

/// Maximum number of channels supported in the FLAC format.
pub const MAX_CHANNELS: usize = 8;

/// Audio frame that contains one sample for each channel.
pub struct Frame {
  /// Information regarding the current audio frame.
  pub header: Header,
  /// Data for each audio channel.
  pub subframes: [Subframe; MAX_CHANNELS],
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
  /// Midpoint and side stereo.
  MidpointSide,
}

/// Numbering scheme used from the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberType {
  /// Frame number of first sample in frame.
  Frame(u32),
  /// Sample number of first sample in frame.
  Sample(u64),
}

/// Information regarding the current audio frame.
#[derive(Debug, PartialEq, Eq)]
pub struct Header {
  /// Number of samples per subframe.
  pub block_size: u32,
  /// Sample rate in hertz (Hz).
  pub sample_rate: u32,
  /// Number of channels that also represent the number of subframes.
  pub channels: u8,
  /// Channel assignment order.
  pub channel_assignment: ChannelAssignment,
  /// Size, in bits, per sample.
  pub bits_per_sample: usize,
  /// Numbering scheme used from the frame.
  pub number: NumberType,
  /// CRC-8 of all header bytes before this crc.
  pub crc: u8,
}

/// End of the audio frame.
///
/// Contains a value that represents the CRC-16 of everything inside the
/// frame before the footer.
#[derive(Debug, PartialEq, Eq)]
pub struct Footer(pub u16);
