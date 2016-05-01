use std::collections::HashMap;
use std::fmt;

/// Data associated with a single metadata block.
#[derive(Debug)]
pub struct Metadata {
  /// Marks whether the current metadata block is the last.
  is_last: bool,
  /// The length, in bytes, of the block being parsed. This does not include
  /// the metadata block header.
  length: u32,
  /// Block data containing one of the eight different types of metadata.
  pub data: Data,
}

/// An enum that represents a metadata block type.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
  /// Represents the current block is stream information.
  StreamInfo,
  /// Represents the current block is padding.
  Padding,
  /// Represents the current block is application data.
  Application,
  /// Represents the current block is a seek table.
  SeekTable,
  /// Represents the current block is a vorbis comment.
  VorbisComment,
  /// Represents the current block is a cue sheet.
  CueSheet,
  /// Represents the current block is a picture.
  Picture,
  /// Represents the current block is unknow.
  Unknown,
}

macro_rules! is_block_type (
  ($(
     $(#[$attr: meta])* ($name: ident) -> $block_type: ident
   )+) => (
    $(
    $(#[$attr])*
    #[inline]
    pub fn $name(&self) -> bool {
      self.data_type() == Type::$block_type
    }
    )+
  );
);

impl Metadata {
  /// Constructs a new `Metadata` struct based on the arguments passed in.
  pub fn new(is_last: bool, length: u32, data: Data) -> Self {
    Metadata {
      is_last: is_last,
      length: length,
      data: data,
    }
  }

  /// Returns whether the current metadata block is the last.
  #[inline]
  pub fn is_last(&self) -> bool {
    self.is_last
  }

  /// Returns the metadata block's type.
  pub fn data_type(&self) -> Type {
    match self.data {
      Data::StreamInfo(_)    => Type::StreamInfo,
      Data::Padding(_)       => Type::Padding,
      Data::Application(_)   => Type::Application,
      Data::SeekTable(_)     => Type::SeekTable,
      Data::VorbisComment(_) => Type::VorbisComment,
      Data::CueSheet(_)      => Type::CueSheet,
      Data::Picture(_)       => Type::Picture,
      Data::Unknown(_)       => Type::Unknown,
    }
  }

  is_block_type! {
    /// Returns true when the current `Metadata` is `StreamInfo`.
    (is_stream_info) -> StreamInfo
    /// Returns true when the current `Metadata` is `Padding`.
    (is_padding) -> Padding
    /// Returns true when the current `Metadata` is `Application`.
    (is_application) -> Application
    /// Returns true when the current `Metadata` is `SeekTable`.
    (is_seek_table) -> SeekTable
    /// Returns true when the current `Metadata` is `VorbisComment`.
    (is_vorbis_comment) -> VorbisComment
    /// Returns true when the current `Metadata` is `CueSheet`.
    (is_cue_sheet) -> CueSheet
    /// Returns true when the current `Metadata` is `Picture`.
    (is_picture) -> Picture
    /// Returns true when the current `Metadata` is `Unknown`.
    (is_unknown) -> Unknown
  }
}

/// General enum that hold all the different metadata block data.
#[derive(Debug, PartialEq, Eq)]
pub enum Data {
  /// Information regarding the entire audio stream.
  StreamInfo(StreamInfo),
  /// Block that represents a number of padded bytes.
  Padding(u32),
  /// Data used by third-party applications.
  Application(Application),
  /// Table of multiple points to seek, or skip, to within the FLAC file.
  SeekTable(Vec<SeekPoint>),
  /// Stores human-readable name/value pairs.
  VorbisComment(VorbisComment),
  /// Stores cue information
  CueSheet(CueSheet),
  /// Stores pictures associated with the FLAC file.
  Picture(Picture),
  /// A type of block data that isn't know or doesn't match the type above.
  Unknown(Vec<u8>),
}

/// Information regarding the entire audio stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamInfo {
  /// Minimum block size, in samples, used in the stream.
  pub min_block_size: u16,
  /// Maximum block size, in samples, used in the stream.
  pub max_block_size: u16,
  /// Minimum frame size, in bytes, used in the stream. May be zero to imply
  /// the value isn't know.
  pub min_frame_size: u32,
  /// Maximum frame size, in bytes, used in the stream. May be zero to imply
  /// the value isn't know.
  pub max_frame_size: u32,
  /// Sample rate in hertz (Hz).
  pub sample_rate: u32,
  /// Number of channels. FLAC supports one to eight channel.
  pub channels: u8,
  /// Bits per sample. FLAC supports four to thirty-two bits per sample.
  pub bits_per_sample: u8,
  /// Total samples in the stream. A value of zero means the number is
  /// unknown.
  pub total_samples: u64,
  /// MD5 signature of the unencoded audio data.
  pub md5_sum: [u8; 16],
}

impl StreamInfo {
  /// Constructs a zeroed out `StreamInfo` struct.
  pub fn new() -> StreamInfo {
    StreamInfo {
      min_block_size: 0,
      max_block_size: 0,
      min_frame_size: 0,
      max_frame_size: 0,
      sample_rate: 0,
      channels: 0,
      bits_per_sample: 0,
      total_samples: 0,
      md5_sum: [0; 16],
    }
  }

  /// Returns true if `min_block_size` and `max_block_size` are different,
  /// otherwise false.
  #[inline]
  pub fn is_varied_block_size(&self) -> bool {
    self.min_block_size != self.max_block_size
  }

  /// Returns true if `min_block_size` and `max_block_size` are equal,
  /// otherwise false.
  #[inline]
  pub fn is_fixed_block_size(&self) -> bool {
    self.min_block_size == self.max_block_size
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = [0; 34];

    bytes[0] = (self.min_block_size >> 8) as u8;
    bytes[1] = self.min_block_size as u8;

    bytes[2] = (self.max_block_size >> 8) as u8;
    bytes[3] = self.max_block_size as u8;

    bytes[4] = (self.min_frame_size >> 16) as u8;
    bytes[5] = (self.min_frame_size >> 8) as u8;
    bytes[6] = self.min_frame_size as u8;

    bytes[7] = (self.max_frame_size >> 16) as u8;
    bytes[8] = (self.max_frame_size >> 8) as u8;
    bytes[9] = self.max_frame_size as u8;

    bytes[10] = (self.sample_rate >> 12) as u8;
    bytes[11] = (self.sample_rate >> 4) as u8;
    bytes[12] = (self.sample_rate << 4) as u8;

    bytes[12] += (self.channels - 1) << 1;
    bytes[12] += (self.bits_per_sample - 1) >> 4;
    bytes[13]  = (self.bits_per_sample - 1) << 4;

    bytes[13] += (self.total_samples >> 32) as u8;
    bytes[14]  = (self.total_samples >> 24) as u8;
    bytes[15]  = (self.total_samples >> 16) as u8;
    bytes[16]  = (self.total_samples >> 8) as u8;
    bytes[17]  = self.total_samples as u8;

    bytes[18..].clone_from_slice(&self.md5_sum);

    bytes.to_vec()
  }
}

/// Data used by third-party applications.
#[derive(Debug, PartialEq, Eq)]
pub struct Application {
  /// Registered application ID.
  pub id: String,
  /// Data used by the third-party application.
  pub data: Vec<u8>,
}

impl Application {
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(4 + self.data.len());

    bytes[0..4].clone_from_slice(self.id.as_bytes());
    bytes[4..].clone_from_slice(&self.data);

    bytes
  }
}

/// Seek, or skip, to a point within the FLAC file.
#[derive(Debug, PartialEq, Eq)]
pub struct SeekPoint {
  /// Sample number of the first sample in the target frame.
  pub sample_number: u64,
  /// Byte offset of the target frame's header.
  pub stream_offset: u64,
  /// Number of samples in the target frame.
  pub frame_samples: u16,
}

impl SeekPoint {
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = [0; 18];

    bytes[0] = (self.sample_number >> 56) as u8;
    bytes[1] = (self.sample_number >> 48) as u8;
    bytes[2] = (self.sample_number >> 40) as u8;
    bytes[3] = (self.sample_number >> 32) as u8;
    bytes[4] = (self.sample_number >> 24) as u8;
    bytes[5] = (self.sample_number >> 16) as u8;
    bytes[6] = (self.sample_number >> 8) as u8;
    bytes[7] = self.sample_number as u8;

    bytes[8]  = (self.stream_offset >> 56) as u8;
    bytes[9]  = (self.stream_offset >> 48) as u8;
    bytes[10] = (self.stream_offset >> 40) as u8;
    bytes[11] = (self.stream_offset >> 32) as u8;
    bytes[12] = (self.stream_offset >> 24) as u8;
    bytes[13] = (self.stream_offset >> 16) as u8;
    bytes[14] = (self.stream_offset >> 8) as u8;
    bytes[15] = self.stream_offset as u8;

    bytes[16] = (self.frame_samples >> 8) as u8;
    bytes[17] = self.frame_samples as u8;

    bytes.to_vec()
  }
}

/// Stores human-readable name/value pairs.
#[derive(Debug, PartialEq, Eq)]
pub struct VorbisComment {
  /// Vendor name.
  pub vendor_string: String,
  /// Comments associated with a name, or category, followed by it's
  /// contents.
  pub comments: HashMap<String, String>,
}

/// Stores cue information.
///
/// Generally for storing information from Compact Disk Digital Audio, but
/// can be used as a cueing mechanism for playback.
#[derive(Debug, PartialEq, Eq)]
pub struct CueSheet {
  /// Media catalog number.
  pub media_catalog_number: String,
  /// Number of lead-in samples.
  pub lead_in: u64,
  /// Whether or not this `CueSheet` corresponds to a Compact Disc.
  pub is_cd: bool,
  /// One or more tracks.
  pub tracks: Vec<CueSheetTrack>,
}

/// Track information inside a cue sheet.
#[derive(Debug, PartialEq, Eq)]
pub struct CueSheetTrack {
  /// Track offset, in samples, relative to the beginning of the FLAC audio
  /// stream.
  pub offset: u64,
  /// Track number.
  pub number: u8,
  /// Twelve digit alphanumeric code.
  pub isrc: String,
  /// Whether the cue sheet track is audio.
  pub is_audio: bool,
  /// Whether the cue sheet track is pre-emphasis.
  pub is_pre_emphasis: bool,
  /// For all tracks except the lead-out track.
  pub indices: Vec<CueSheetTrackIndex>,
}

/// An index point within a track, inside of a cue sheet.
#[derive(Debug, PartialEq, Eq)]
pub struct CueSheetTrackIndex {
  /// Offset, in samples, relative to the track offset of the index point.
  pub offset: u64,
  /// Index point number.
  pub number: u8,
}

/// Stores pictures associated with the FLAC file.
///
/// More than likely these pictures will be cover art, but you can have more
/// than one within a file, which are distinguished by `PictureType`and it's
/// mime type string.
#[derive(Debug, PartialEq, Eq)]
pub struct Picture {
  /// Picture type, based on the ID3v2 APIC frame.
  pub picture_type: PictureType,
  /// Multipurpose Internet Mail Extensions (MIME) type.
  pub mime_type: String,
  /// A string describing the picture.
  pub description: String,
  /// Width of the picture in pixels.
  pub width: u32,
  /// Height of the picture in pixels.
  pub height: u32,
  /// Color depth of the picture in bits-per-pixel.
  pub depth: u32,
  /// Number of colors used.
  pub colors: u32,
  /// Binary picture data.
  pub data: Vec<u8>,
}

/// The picture type according to the ID3v2 attached picture frame.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PictureType {
  /// Other picture type not categorized in this enum.
  Other,
  /// 32x32 pixels 'file icon'.
  FileIconStandard,
  /// Other, or non-standard, file icon.
  FileIcon,
  /// Cover (front).
  FrontCover,
  /// Cover (back).
  BackCover,
  /// Leaflet page.
  LeafletPage,
  /// Media, like label side of a CD.
  Media,
  /// Lead artist, lead performer, or soloist.
  LeadArtist,
  /// Artist or performer.
  Artist,
  /// Conductor.
  Conductor,
  /// Band or orchestra.
  Band,
  /// Composer.
  Composer,
  /// Lyricist or text writer.
  Lyricist,
  /// Recording location.
  RecordingLocation,
  /// During recording.
  DuringRecording,
  /// During performance.
  DuringPerformance,
  /// Movie, or video, screen capture.
  VideoScreenCapture,
  /// A bright colored fish.
  Fish,
  /// Illustration.
  Illustration,
  /// Band, or artist, logotype.
  BandLogo,
  /// Publisher, or studio, logotype.
  PublisherLogo,
}

impl fmt::Display for PictureType {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "{}", match *self {
      PictureType::Other              => "Other",
      PictureType::FileIconStandard   => "File Icon (standard)",
      PictureType::FileIcon           => "File Icon",
      PictureType::FrontCover         => "Cover (front)",
      PictureType::BackCover          => "Cover (back)",
      PictureType::LeafletPage        => "Leaflet Page",
      PictureType::Media              => "Media",
      PictureType::LeadArtist         => "Lead Artist",
      PictureType::Artist             => "Arist",
      PictureType::Conductor          => "Conductor",
      PictureType::Band               => "Band",
      PictureType::Composer           => "Composer",
      PictureType::Lyricist           => "Lyricist",
      PictureType::RecordingLocation  => "Recoding Location",
      PictureType::DuringRecording    => "During Recording",
      PictureType::DuringPerformance  => "During Performance",
      PictureType::VideoScreenCapture => "Video Screen Capture",
      PictureType::Fish               => "Fish",
      PictureType::Illustration       => "Illustration",
      PictureType::BandLogo           => "Band Logo",
      PictureType::PublisherLogo      => "Publisher Logo",
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_varied_block_size() {
    let mut info = StreamInfo::new();

    info.min_block_size = 512;
    info.max_block_size = 1024;

    assert!(info.is_varied_block_size());

    info.min_block_size = 2048;
    info.max_block_size = 2048;

    assert!(!info.is_varied_block_size());
  }

  #[test]
  fn test_is_fixed_block_size() {
    let mut info = StreamInfo::new();

    info.min_block_size = 512;
    info.max_block_size = 512;

    assert!(info.is_fixed_block_size());

    info.min_block_size = 1024;
    info.max_block_size = 2048;

    assert!(!info.is_fixed_block_size());
  }
}
