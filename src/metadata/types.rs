use std::collections::HashMap;
use std::fmt;
use std::io;

use utility::WriteExtension;

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

  pub fn to_bytes(&self) -> Vec<u8> {
    let byte = if self.is_last {
      0b10000000
    } else {
      0b00000000
    };

    match self.data {
      Data::StreamInfo(ref stream_info)       => {
        let length    = stream_info.bytes_len();
        let mut bytes = Vec::with_capacity(4 + length);

        bytes.write_u8(byte + 0);

        bytes.write_be_u24(length as u32);

        stream_info.to_bytes(&mut bytes);

        bytes
      }
      Data::Padding(_length)                  => {
        use std::io::Write;

        let length    = _length as usize;
        let mut bytes = Vec::with_capacity(4 + length);
        let padding   = vec![0; length];

        bytes.write_u8(byte + 1);

        bytes.write_be_u24(length as u32);

        bytes.write_all(&padding);

        bytes
      }
      Data::Application(ref application)      => {
        let length    = application.bytes_len();
        let mut bytes = Vec::with_capacity(4 + length);

        bytes.write_u8(byte + 2);

        bytes.write_be_u24(length as u32);

        application.to_bytes(&mut bytes);

        bytes
      }
      Data::SeekTable(ref seek_points)        => {
        let length    = seek_points.iter().fold(0, |result, seek_point|
                          result + seek_point.bytes_len());
        let mut bytes = Vec::with_capacity(4 + length);

        bytes.write_u8(byte + 3);

        bytes.write_be_u24(length as u32);

        for seek_point in seek_points {
          seek_point.to_bytes(&mut bytes);
        }

        bytes
      }
      Data::VorbisComment(ref vorbis_comment) => {
        let length    = vorbis_comment.bytes_len();
        let mut bytes = Vec::with_capacity(4 + length);

        bytes.write_u8(byte + 4);

        bytes.write_be_u24(length as u32);

        vorbis_comment.to_bytes(&mut bytes);

        bytes
      }
      Data::CueSheet(ref cue_sheet)           => {
        let length    = cue_sheet.bytes_len();
        let mut bytes = Vec::with_capacity(4 + length);

        bytes.write_u8(byte + 5);

        bytes.write_be_u24(length as u32);

        cue_sheet.to_bytes(&mut bytes);

        bytes
      }
      Data::Picture(ref picture)              => {
        let length    = picture.bytes_len();
        let mut bytes = vec![0; 4 + length];

        bytes[0] = byte + 6;

        bytes[1] = (length >> 16) as u8;
        bytes[2] = (length >> 8) as u8;
        bytes[3] = length as u8;

        picture.to_bytes_buffer(&mut bytes[4..]);

        bytes
      }
      Data::Unknown(ref unknown)              => {
        let length    = unknown.len();
        let mut bytes = vec![0; 4 + length];

        bytes[0] = byte + 7;

        bytes[1] = (length >> 16) as u8;
        bytes[2] = (length >> 8) as u8;
        bytes[3] = length as u8;

        bytes[4..].clone_from_slice(&unknown);

        bytes
      },
    }
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

  #[inline]
  pub fn bytes_len(&self) -> usize {
    34
  }

  pub fn to_bytes<Write: io::Write>(&self, buffer: &mut Write)
                                    -> io::Result<()> {
    try!(buffer.write_be_u16(self.min_block_size));
    try!(buffer.write_be_u16(self.max_block_size));

    try!(buffer.write_be_u24(self.min_frame_size));
    try!(buffer.write_be_u24(self.max_frame_size));

    let bytes = [
      (self.sample_rate >> 12) as u8,
      (self.sample_rate >> 4) as u8,

      ((self.sample_rate << 4) as u8) | ((self.channels - 1) << 1) |
      ((self.bits_per_sample - 1) >> 4),

      ((self.bits_per_sample - 1) << 4) | ((self.total_samples >> 32) as u8),
    ];

    try!(buffer.write_all(&bytes));

    try!(buffer.write_be_u32(self.total_samples as u32));

    buffer.write_all(&self.md5_sum)
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
  #[inline]
  pub fn bytes_len(&self) -> usize {
    4 + self.data.len()
  }

  pub fn to_bytes<Write: io::Write>(&self, buffer: &mut Write)
                                    -> io::Result<()> {
    try!(buffer.write_all(&self.id.as_bytes()));

    buffer.write_all(&self.data)
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
  pub fn bytes_len(&self) -> usize {
    18
  }

  pub fn to_bytes<Write: io::Write>(&self, buffer: &mut Write)
                                    -> io::Result<()> {
    try!(buffer.write_be_u64(self.sample_number));

    try!(buffer.write_be_u64(self.stream_offset));

    buffer.write_be_u16(self.frame_samples)
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
  pub fn bytes_len(&self) -> usize {
    let vendor_bytes   = self.vendor_string.as_bytes();
    let vendor_length  = vendor_bytes.len();

     self.comments.iter().fold(0, |result, (k, v)| {
       let k_length = k.as_bytes().len();
       let v_length = v.as_bytes().len();

       result + k_length + 5 + v_length
     }) + 8 + vendor_length
  }

  pub fn to_bytes<Write: io::Write>(&self, buffer: &mut Write)
                                    -> io::Result<()> {
    let vendor_bytes   = self.vendor_string.as_bytes();
    let vendor_length  = vendor_bytes.len();
    let comments_count = self.comments.len();

    try!(buffer.write_le_u32(vendor_length as u32));
    try!(buffer.write_all(vendor_bytes));

    try!(buffer.write_le_u32(comments_count as u32));

    for (key, value) in &self.comments {
      let key_bytes    = key.as_bytes();
      let key_length   = key_bytes.len();
      let value_bytes  = value.as_bytes();
      let value_length = value_bytes.len();
      let length       = key_length + value_length + 1;

      try!(buffer.write_le_u32(length as u32));

      try!(buffer.write_all(key_bytes));
      try!(buffer.write_u8(b'='));


      try!(buffer.write_all(value_bytes));
    }

    Ok(())
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
  #[inline]
  pub fn bytes_len(&self) -> usize {
    self.tracks.iter().fold(0, |result, track| {
      result + track.bytes_len()
    }) + 396
  }

  pub fn to_bytes<Write: io::Write>(&self, mut buffer: Write)
                                   -> io::Result<()> {
    let mut flag   = 0;
    let tracks_len = self.tracks.len();

    try!(buffer.write_all(self.media_catalog_number.as_bytes()));

    try!(buffer.write_be_u64(self.lead_in));

    if self.is_cd {
      flag |= 0b10000000;
    }

    try!(buffer.write_u8(flag));

    try!(buffer.write_all(&[0; 258]));

    try!(buffer.write_u8(tracks_len as u8));

    for track in &self.tracks {
      try!(track.to_bytes(&mut buffer));
    }

    Ok(())
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

  pub fn to_bytes<Write: io::Write>(&self, buffer: &mut Write)
                                   -> io::Result<()> {
    let num_indices = self.indices.len();
    let mut flags   = 0;

    try!(buffer.write_be_u64(self.offset));

    try!(buffer.write_u8(self.number));

    try!(buffer.write_all(self.isrc.as_bytes()));

    if !self.is_audio {
      flags |= 0b10000000;
    }

    if self.is_pre_emphasis {
      flags |= 0b01000000;
    }

    try!(buffer.write_u8(flags));

    try!(buffer.write_all(&[0; 13]));

    try!(buffer.write_u8(num_indices as u8));

    for indice in &self.indices {
      try!(indice.to_bytes(buffer));
    }

    Ok(())
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
  #[inline]
  pub fn bytes_len(&self) -> usize {
    12
  }

  pub fn to_bytes<Write: io::Write>(&self, buffer: &mut Write)
                                    -> io::Result<()> {
    try!(buffer.write_be_u64(self.offset));

    try!(buffer.write_u8(self.number));

    buffer.write_all(&[0; 3])
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
  pub fn bytes_len(&self) -> usize {
    let mime_type       = self.mime_type.as_bytes();
    let mime_type_len   = mime_type.len();
    let description     = self.description.as_bytes();
    let description_len = description.len();
    let data_len        = self.data.len();

    32 + mime_type_len + description_len + data_len
  }

  pub fn to_bytes<Write: io::Write>(&self, buffer: &mut Write)
                                    -> io::Result<()> {
    let mime_type       = self.mime_type.as_bytes();
    let mime_type_len   = mime_type.len();
    let description     = self.description.as_bytes();
    let description_len = description.len();
    let data_len        = self.data.len();

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


    try!(buffer.write_be_u32(picture_type));

    try!(buffer.write_be_u32(mime_type_len as u32));
    try!(buffer.write_all(mime_type));

    try!(buffer.write_be_u32(description_len as u32));
    try!(buffer.write_all(description));

    try!(buffer.write_be_u32(self.width));
    try!(buffer.write_be_u32(self.height));
    try!(buffer.write_be_u32(self.depth));
    try!(buffer.write_be_u32(self.colors));

    try!(buffer.write_be_u32(data_len as u32));
    buffer.write_all(&self.data)
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
      let stream_info = StreamInfo {
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

      let input  = Metadata::new(false, 34, Data::StreamInfo(stream_info));
      let result = b"\0\0\0\x22\x12\0\x12\0\0\0\x0e\0\0\x10\x01\xf4\x02\x70\
                     \0\x01\x38\x80\xa0\x42\x23\x7c\x54\x93\xfd\xb9\x65\x6b\
                     \x94\xa8\x36\x08\xd1\x1a";

      assert_eq!(&input.to_bytes()[..], &result[..]);
    }

    {
      let stream_info = StreamInfo {
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

      let input  = Metadata::new(true, 34, Data::StreamInfo(stream_info));
      let result = b"\x80\0\0\x22\x10\0\x10\0\0\x0a\xab\0\x53\x05\x0b\xb8\
                     \x03\x70\0\x9b\x8f\x4a\xc6\x16\x1b\x2b\xb3\xf8\x1c\xa6\
                     \x72\x79\x1d\x96\xf0\x9d\x0b\x0c";

      assert_eq!(&input.to_bytes()[..], &result[..]);
    }
  }

  #[test]
  fn test_padding_to_bytes() {
    let input  = Metadata::new(false, 10, Data::Padding(10));
    let result = b"\x01\0\0\x0a\0\0\0\0\0\0\0\0\0\0";

    assert_eq!(&input.to_bytes()[..], &result[..]);
  }

  #[test]
  fn test_application_to_bytes() {
    {
      let application = Application {
        id: "fake".to_owned(),
        data: vec![],
      };

      let input  = Metadata::new(true, 4, Data::Application(application));
      let result = b"\x82\0\0\x04fake";

      assert_eq!(&input.to_bytes()[..], &result[..]);
    }

    {
      let application = Application {
        id: "riff".to_owned(),
        data: b"fake data"[..].to_owned(),
      };

      let input  = Metadata::new(false, 13, Data::Application(application));
      let result = b"\x02\0\0\x0drifffake data";

      assert_eq!(&input.to_bytes()[..], &result[..]);
    }
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

    let input  = Metadata::new(true, 90, Data::SeekTable(seek_points));
    let result = b"\x83\0\0\x5a\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x12\0\0\0\0\
                   \0\0\0\x12\0\0\0\0\0\0\0\0\x0e\x04\xf8\xff\xff\xff\xff\
                   \xff\xff\xff\xff\0\0\0\0\0\0\0\0\0\0\xff\xff\xff\xff\xff\
                   \xff\xff\xff\0\0\0\0\0\0\0\0\0\0\xff\xff\xff\xff\xff\xff\
                   \xff\xff\0\0\0\0\0\0\0\0\0\0";


    assert_eq!(&input.to_bytes()[..], &result[..]);
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

    let mut result = vec![0; 207];
    let mut offset = 44;

    result[0..offset].clone_from_slice(
      b"\x04\0\0\xcb\x20\0\0\0reference libFLAC 1.1.3 20060805\x06\0\0\0");

    for key in comments.keys() {
      let bytes = if key == "REPLAYGAIN_TRACK_PEAK" {
        &b"\x20\0\0\0REPLAYGAIN_TRACK_PEAK=0.99996948"[..]
      } else if key == "REPLAYGAIN_TRACK_GAIN" {
        &b"\x1e\0\0\0REPLAYGAIN_TRACK_GAIN=-7.89 dB"[..]
      } else if key == "REPLAYGAIN_ALBUM_PEAK" {
        &b"\x20\0\0\0REPLAYGAIN_ALBUM_PEAK=0.99996948"[..]
      } else if key == "REPLAYGAIN_ALBUM_GAIN" {
        &b"\x1e\0\0\0REPLAYGAIN_ALBUM_GAIN=-7.89 dB"[..]
      } else if key == "artist" {
        &b"\x08\0\0\0artist=1"[..]
      } else if key == "title" {
        &b"\x07\0\0\0title=2"[..]
      } else {
        &b""[..]
      };

      let bytes_len = bytes.len();

      result[offset..(offset + bytes_len)].clone_from_slice(bytes);

      offset += bytes_len;
    }

    let vorbis_comment = VorbisComment{
      vendor_string: "reference libFLAC 1.1.3 20060805".to_owned(),
      comments: comments,
    };

    let input = Metadata::new(false, 203,
      Data::VorbisComment(vorbis_comment));

    assert_eq!(&input.to_bytes()[..], &result[..]);
  }

  #[test]
  fn test_cue_sheet_to_bytes() {
    let cue_sheet = CueSheet {
      media_catalog_number: "1234567890123\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                             \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                             \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                             \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                             \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                             \0\0\0\0\0\0".to_owned(),
      lead_in: 88200,
      is_cd: true,
      tracks: vec![
        CueSheetTrack {
          offset: 0,
          number: 1,
          isrc: "\0\0\0\0\0\0\0\0\0\0\0\0".to_owned(),
          is_audio: true,
          is_pre_emphasis: false,
          indices: vec![
            CueSheetTrackIndex {
              offset: 0,
              number: 1,
            },
            CueSheetTrackIndex {
              offset: 588,
              number: 2,
            }
          ],
        },
        CueSheetTrack {
          offset: 2940,
          number: 2,
          isrc: "\0\0\0\0\0\0\0\0\0\0\0\0".to_owned(),
          is_audio: true,
          is_pre_emphasis: false,
          indices: vec![
            CueSheetTrackIndex {
              offset: 0,
              number: 1,
            }
          ],
        },
        CueSheetTrack {
          offset: 5880,
          number: 170,
          isrc: "\0\0\0\0\0\0\0\0\0\0\0\0".to_owned(),
          is_audio: true,
          is_pre_emphasis: false,
          indices: vec![],
        },
      ],
    };

    let input  = Metadata::new(true, 540, Data::CueSheet(cue_sheet));
    let result = b"\x85\0\x02\x1c1234567890123\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x58\x88\
                   \x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\x03\0\0\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x02\0\0\0\0\0\0\0\0\
                   \x01\0\0\0\0\0\0\0\0\0\x02\x4c\x02\0\0\0\0\0\0\0\0\0\x0b\
                   \x7c\x02\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\x01\0\0\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\0\0\x16\xf8\
                   \xaa\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0";

    assert_eq!(&input.to_bytes()[..], &result[..]);
  }

  #[test]
  fn test_picture_to_bytes() {
    let picture = Picture {
      picture_type: PictureType::Other,
      mime_type: "image/png".to_owned(),
      description: String::new(),
      width: 0,
      height: 0,
      depth: 0,
      colors: 0,
      data: vec![],
    };

    let input  = Metadata::new(false, 41, Data::Picture(picture));
    let result = b"\x06\0\0\x29\0\0\0\0\0\0\0\x09image/png\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

    assert_eq!(&input.to_bytes()[..], &result[..]);
  }

  #[test]
  fn test_unknown_to_bytes() {
    let unknown = Data::Unknown(b"random data that won't really be parsed \
                                  anyway."[..].to_owned());
    let input   = Metadata::new(true, 47, unknown);
    let result  = b"\x87\0\0\x2frandom data that won't really be parsed \
                    anyway.";

    assert_eq!(&input.to_bytes()[..], &result[..]);
  }
}
