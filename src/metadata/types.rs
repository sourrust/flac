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
    let mut bytes = vec![0; (4 + self.data.len())];

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

    self.to_bytes_buffer(&mut bytes);

    bytes.to_vec()
  }

  pub fn to_bytes_buffer(&self, bytes: &mut [u8]) {
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

impl VorbisComment {
  pub fn to_bytes(&self) -> Vec<u8> {
    let vendor_bytes   = self.vendor_string.as_bytes();
    let vendor_length  = vendor_bytes.len();
    let comments_count = self.comments.len();
    let capacity       = 8 + vendor_length +
                         self.comments.iter().fold(0, |result, (k, v)| {
                           let k_length = k.as_bytes().len();
                           let v_length = v.as_bytes().len();

                           result + k_length + 1 + v_length
                         });

    let mut bytes = Vec::with_capacity(capacity);

    bytes[0] = vendor_length as u8;
    bytes[1] = (vendor_length >> 8) as u8;
    bytes[2] = (vendor_length >> 16) as u8;
    bytes[3] = (vendor_length >> 24) as u8;

    bytes[4..(4 + vendor_length)].clone_from_slice(vendor_bytes);

    bytes[vendor_length + 4] = comments_count as u8;
    bytes[vendor_length + 5] = (comments_count >> 8) as u8;
    bytes[vendor_length + 6] = (comments_count >> 16) as u8;
    bytes[vendor_length + 7] = (comments_count >> 24) as u8;

    let mut offset = vendor_length + 8;

    for (key, value) in &self.comments {
      let key_length   = key.len();
      let key_bytes    = key.as_bytes();
      let value_length = value.len();
      let value_bytes  = value.as_bytes();
      let length       = key_length + value_length + 1;

      bytes[offset + 0] = length as u8;
      bytes[offset + 1] = (length >> 8) as u8;
      bytes[offset + 2] = (length >> 16) as u8;
      bytes[offset + 3] = (length >> 24) as u8;

      offset += 4;

      bytes[offset..(offset + key_length)].clone_from_slice(key_bytes);
      bytes[offset + key_length] = b'=';

      offset += key_length + 1;

      bytes[offset..(offset + value_length)].clone_from_slice(value_bytes);

      offset += value_length;
    }

    bytes
  }
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

impl CueSheet {
  pub fn to_bytes(&self) -> Vec<u8> {
    let tracks_bytes = self.tracks.iter().fold(0, |result, track| {
      result + track.bytes_len()
    });

    let mut bytes  = Vec::with_capacity(396 * tracks_bytes);
    let mut flag   = 0;
    let tracks_len = self.tracks.len();

    bytes[0..128].clone_from_slice(self.media_catalog_number.as_bytes());

    bytes[128] = (self.lead_in >> 56) as u8;
    bytes[129] = (self.lead_in >> 48) as u8;
    bytes[130] = (self.lead_in >> 40) as u8;
    bytes[131] = (self.lead_in >> 32) as u8;
    bytes[132] = (self.lead_in >> 24) as u8;
    bytes[133] = (self.lead_in >> 16) as u8;
    bytes[134] = (self.lead_in >> 8) as u8;
    bytes[135] = self.lead_in as u8;

    if self.is_cd {
      flag |= 0b10000000;
    }

    bytes[136] = flag;

    bytes[137..395].clone_from_slice(&[0; 258]);

    bytes[395] = tracks_len as u8;

    let mut offset = 396;

    for track in &self.tracks {
      let len = track.bytes_len();

      track.to_bytes_buffer(&mut bytes[offset..(offset + len)]);

      offset += len;
    }

    bytes
  }
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

impl CueSheetTrack {
  pub fn bytes_len(&self) -> usize {
    let num_indices = self.indices.len();

    36 + num_indices * 12
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(self.bytes_len());

    self.to_bytes_buffer(&mut bytes);

    bytes
  }

  pub fn to_bytes_buffer(&self, bytes: &mut [u8]) {
    let num_indices = self.indices.len();
    let mut flags   = 0;

    bytes[0] = (self.offset >> 56) as u8;
    bytes[1] = (self.offset >> 48) as u8;
    bytes[2] = (self.offset >> 40) as u8;
    bytes[3] = (self.offset >> 32) as u8;
    bytes[4] = (self.offset >> 24) as u8;
    bytes[5] = (self.offset >> 16) as u8;
    bytes[6] = (self.offset >> 8) as u8;
    bytes[7] = self.offset as u8;

    bytes[8] = self.number;

    bytes[9..21].clone_from_slice(self.isrc.as_bytes());

    if !self.is_audio {
      flags |= 0b10000000;
    }

    if self.is_pre_emphasis {
      flags |= 0b01000000;
    }

    bytes[21] = flags;

    bytes[22..35].clone_from_slice(&[0; 13]);

    bytes[35] = num_indices as u8;

    let mut offset = 36;

    for indice in &self.indices {
      indice.to_bytes_buffer(&mut bytes[offset..(offset + 12)]);

      offset += 12;
    }
  }
}

/// An index point within a track, inside of a cue sheet.
#[derive(Debug, PartialEq, Eq)]
pub struct CueSheetTrackIndex {
  /// Offset, in samples, relative to the track offset of the index point.
  pub offset: u64,
  /// Index point number.
  pub number: u8,
}

impl CueSheetTrackIndex {
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = [0; 12];

    self.to_bytes_buffer(&mut bytes);

    bytes.to_vec()
  }

  pub fn to_bytes_buffer(&self, bytes: &mut [u8]) {
    bytes[0] = (self.offset >> 56) as u8;
    bytes[1] = (self.offset >> 48) as u8;
    bytes[2] = (self.offset >> 40) as u8;
    bytes[3] = (self.offset >> 32) as u8;
    bytes[4] = (self.offset >> 24) as u8;
    bytes[5] = (self.offset >> 16) as u8;
    bytes[6] = (self.offset >> 8) as u8;
    bytes[7] = self.offset as u8;

    bytes[8] = self.number;

    bytes[9..].clone_from_slice(&[0; 3]);
  }
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

impl Picture {
  pub fn to_bytes(&self) -> Vec<u8> {
    let mime_type       = self.mime_type.as_bytes();
    let mime_type_len   = mime_type.len();
    let description     = self.description.as_bytes();
    let description_len = description.len();
    let data_len        = self.data.len();
    let extra_bytes     = mime_type_len + description_len + data_len;

    let mut bytes = Vec::with_capacity(32 + extra_bytes);

    let picture_type: u32 = match self.picture_type {
      PictureType::Other              => 0,
      PictureType::FileIconStandard   => 1,
      PictureType::FileIcon           => 2,
      PictureType::FrontCover         => 3,
      PictureType::BackCover          => 4,
      PictureType::LeafletPage        => 5,
      PictureType::Media              => 6,
      PictureType::LeadArtist         => 7,
      PictureType::Artist             => 8,
      PictureType::Conductor          => 9,
      PictureType::Band               => 10,
      PictureType::Composer           => 11,
      PictureType::Lyricist           => 12,
      PictureType::RecordingLocation  => 13,
      PictureType::DuringRecording    => 14,
      PictureType::DuringPerformance  => 15,
      PictureType::VideoScreenCapture => 16,
      PictureType::Fish               => 17,
      PictureType::Illustration       => 18,
      PictureType::BandLogo           => 19,
      PictureType::PublisherLogo      => 20,
    };

    bytes[0] = (picture_type >> 24) as u8;
    bytes[1] = (picture_type >> 16) as u8;
    bytes[2] = (picture_type >> 8) as u8;
    bytes[3] = picture_type as u8;

    bytes[4] = (mime_type_len >> 24) as u8;
    bytes[5] = (mime_type_len >> 16) as u8;
    bytes[6] = (mime_type_len >> 8) as u8;
    bytes[7] = mime_type_len as u8;

    let mut offset = 8 + mime_type_len;

    bytes[8..offset].clone_from_slice(mime_type);

    bytes[offset]     = (description_len >> 24) as u8;
    bytes[offset + 1] = (description_len >> 16) as u8;
    bytes[offset + 2] = (description_len >> 8) as u8;
    bytes[offset + 3] = description_len as u8;

    offset += 4;

    bytes[offset..(offset + description_len)].clone_from_slice(description);

    offset += description_len;

    bytes[offset]     = (self.width >> 24) as u8;
    bytes[offset + 1] = (self.width >> 16) as u8;
    bytes[offset + 2] = (self.width >> 8) as u8;
    bytes[offset + 3] = self.width as u8;

    offset += 4;

    bytes[offset]     = (self.height >> 24) as u8;
    bytes[offset + 1] = (self.height >> 16) as u8;
    bytes[offset + 2] = (self.height >> 8) as u8;
    bytes[offset + 3] = self.height as u8;

    offset += 4;

    bytes[offset]     = (self.depth >> 24) as u8;
    bytes[offset + 1] = (self.depth >> 16) as u8;
    bytes[offset + 2] = (self.depth >> 8) as u8;
    bytes[offset + 3] = self.depth as u8;

    offset += 4;

    bytes[offset]     = (self.colors >> 24) as u8;
    bytes[offset + 1] = (self.colors >> 16) as u8;
    bytes[offset + 2] = (self.colors >> 8) as u8;
    bytes[offset + 3] = self.colors as u8;

    offset += 4;

    bytes[offset]     = (data_len >> 24) as u8;
    bytes[offset + 1] = (data_len >> 16) as u8;
    bytes[offset + 2] = (data_len >> 8) as u8;
    bytes[offset + 3] = data_len as u8;

    offset += 4;

    bytes[offset..(offset + data_len)].clone_from_slice(&self.data);

    bytes
  }
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

  use std::collections::HashMap;

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

  #[test]
  fn test_stream_info_to_bytes() {
    {
      let input = StreamInfo {
        min_block_size: 4608,
        max_block_size: 4608,
        min_frame_size: 14,
        max_frame_size: 16,
        sample_rate: 8000,
        channels: 2,
        bits_per_sample: 8,
        total_samples: 80000,
        md5_sum: [ 0xa0, 0x42, 0x23, 0x7c, 0x54, 0x93, 0xfd, 0xb9, 0x65, 0x6b
                 , 0x94, 0xa8, 0x36, 0x08, 0xd1, 0x1a
                 ],
      };

      let result = b"\x12\0\x12\0\0\0\x0e\0\0\x10\x01\xf4\x02\x70\0\x01\x38\
                    \x80\xa0\x42\x23\x7c\x54\x93\xfd\xb9\x65\x6b\x94\xa8\x36\
                    \x08\xd1\x1a";

      assert_eq!(&input.to_bytes()[..], &result[..]);
    }

    {
      let input = StreamInfo {
        min_block_size: 4096,
        max_block_size: 4096,
        min_frame_size: 2731,
        max_frame_size: 21253,
        sample_rate: 48000,
        channels: 2,
        bits_per_sample: 24,
        total_samples: 10194762,
        md5_sum: [ 0xc6, 0x16, 0x1b, 0x2b, 0xb3, 0xf8, 0x1c, 0xa6, 0x72, 0x79
                 , 0x1d, 0x96, 0xf0, 0x9d, 0x0b, 0x0c
                 ],
      };

      let result = b"\x10\0\x10\0\0\x0a\xab\0\x53\x05\x0b\xb8\x03\x70\0\x9b\
                     \x8f\x4a\xc6\x16\x1b\x2b\xb3\xf8\x1c\xa6\x72\x79\x1d\x96\
                     \xf0\x9d\x0b\x0c";

      assert_eq!(&input.to_bytes()[..], &result[..]);
    }
  }

  #[test]
  fn test_application_to_bytes() {
    let inputs = [
      Application {
        id: "fake".to_owned(),
        data: vec![],
      },
      Application {
        id: "riff".to_owned(),
        data: b"fake data"[..].to_owned(),
      }
    ];

    let results = [&b"fake"[..], &b"rifffake data"[..]];

    assert_eq!(&inputs[0].to_bytes()[..], results[0]);
    assert_eq!(&inputs[1].to_bytes()[..], results[1]);
  }

  #[test]
  fn test_seek_table_to_bytes() {
    let seek_points = vec![
      SeekPoint {
        sample_number: 0,
        stream_offset: 0,
        frame_samples: 4608,
      },
      SeekPoint {
        sample_number: 4608,
        stream_offset: 14,
        frame_samples: 1272,
      },
      SeekPoint {
        sample_number: 0xffffffffffffffff,
        stream_offset: 0,
        frame_samples: 0,
      },
      SeekPoint {
        sample_number: 0xffffffffffffffff,
        stream_offset: 0,
        frame_samples: 0,
      },
      SeekPoint {
        sample_number: 0xffffffffffffffff,
        stream_offset: 0,
        frame_samples: 0,
      }
    ];

    let result = b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x12\0\0\0\0\0\0\0\x12\0\0\
                   \0\0\0\0\0\0\x0e\x04\xf8\xff\xff\xff\xff\xff\xff\xff\xff\0\
                   \0\0\0\0\0\0\0\0\0\xff\xff\xff\xff\xff\xff\xff\xff\0\0\0\0\
                   \0\0\0\0\0\0\xff\xff\xff\xff\xff\xff\xff\xff\0\0\0\0\0\0\0\
                   \0\0\0";

    let mut bytes = [0; 90];

    for i in 0..5 {
      let seek_point = &seek_points[i];
      let start      = 18 * i;
      let end        = 18 * (i + 1);

      seek_point.to_bytes_buffer(&mut bytes[start..end])
    }

    assert_eq!(&bytes[..], &result[..]);
  }

  #[test]
  fn test_vorbis_comment_to_bytes() {
    let mut comments = HashMap::with_capacity(6);

    comments.insert("REPLAYGAIN_TRACK_PEAK".to_owned(),
                    "0.99996948".to_owned());
    comments.insert("REPLAYGAIN_TRACK_GAIN".to_owned(),
                    "-7.89 dB".to_owned());
    comments.insert("REPLAYGAIN_ALBUM_PEAK".to_owned(),
                    "0.99996948".to_owned());
    comments.insert("REPLAYGAIN_ALBUM_GAIN".to_owned(),
                    "-7.89 dB".to_owned());
    comments.insert("artist".to_owned(), "1".to_owned());
    comments.insert("title".to_owned(), "2".to_owned());

    let input = VorbisComment{
      vendor_string: "reference libFLAC 1.1.3 20060805".to_owned(),
      comments: comments,
    };

    let result = b"\x20\0\0\0reference libFLAC 1.1.3 20060805\x06\0\0\0\
                   \x20\0\0\0REPLAYGAIN_TRACK_PEAK=0.99996948\
                   \x1e\0\0\0REPLAYGAIN_TRACK_GAIN=-7.89 dB\
                   \x20\0\0\0REPLAYGAIN_ALBUM_PEAK=0.99996948\
                   \x1e\0\0\0REPLAYGAIN_ALBUM_GAIN=-7.89 dB\
                   \x08\0\0\0artist=1\x07\0\0\0title=2";

    let bytes = input.to_bytes();
    println!("input: {}", bytes.len());
    println!("result: {}", result.len());
    assert_eq!(&bytes[..], &result[..]);
  }
}
