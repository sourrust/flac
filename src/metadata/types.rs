pub struct Block<'a> {
  pub is_last: bool,
  pub length: u32,
  pub data: BlockData<'a>,
}

pub enum BlockData<'a> {
  StreamInfo(StreamInfo<'a>),
  Padding(u32),
  Application(Application<'a>),
  SeekTable(Vec<SeekPoint>),
  VorbisComment(VorbisComment<'a>),
  CueSheet(CueSheet<'a>),
  Picture(Picture<'a>),
  Unknown(&'a [u8]),
}

/// Information regarding the entire audio stream
pub struct StreamInfo<'a> {
  pub min_block_size: u16,
  pub max_block_size: u16,
  pub min_frame_size: u32,
  pub max_frame_size: u32,
  pub sample_rate: u32,
  pub channels: u8,
  pub bits_per_sample: u8,
  pub total_samples: u64,
  pub md5_sum: &'a [u8],
}

/// Data used by third-party applications
pub struct Application<'a> {
  pub id: &'a str,
  pub data: &'a [u8],
}

/// Seek, or skip, to a point within the FLAC file
pub struct SeekPoint {
  pub sample_number: u64,
  pub stream_offset: u64,
  pub frame_samples: u16,
}

/// Stores human-readable name/value pairs
pub struct VorbisComment<'a> {
  pub vendor_string: &'a str,
  pub comments: Vec<&'a str>,
}

/// Stores cue information
///
/// Generally for storing information from Compact Disk Digital Audio, but
/// can be used as a cueing mechanism for playback.
pub struct CueSheet<'a> {
  pub media_catalog_number: &'a str,
  pub lead_in: u64,
  pub is_cd: bool,
  pub tracks: Vec<CueSheetTrack<'a>>,
}

pub struct CueSheetTrack<'a> {
  pub offset: u64,
  pub number: u8,
  pub isrc: &'a str,
  pub isnt_audio: bool,
  pub is_pre_emphasis: bool,
  pub indices: Vec<CueSheetTrackIndex>,
}

pub struct CueSheetTrackIndex {
  pub offset: u64,
  pub number: u8,
}

/// Stores pictures associated with a FLAC file
///
/// More than likely these pictures will be cover art, but you can have more
/// than one within a file, which are distinguished by `PictureType`and it's
/// mime type string.
pub struct Picture<'a> {
  pub picture_type: PictureType,
  pub mime_type: &'a str,
  pub description: &'a str,
  pub width: u32,
  pub height: u32,
  pub depth: u32,
  pub colors: u32,
  pub data: &'a [u8],
}

pub enum PictureType {
  Other,
  FileIconStandard,
  FileIcon,
  FrontCover,
  BackCover,
  LeafletPage,
  Media,
  LeadArtist,
  Artist,
  Conductor,
  Band,
  Composer,
  Lyricist,
  RecordingLocation,
  DuringRecording,
  DuringPerformace,
  VideoScreenCapture,
  Fish,
  Illustration,
  BandLogoType,
  PublisherLogoType,
}
