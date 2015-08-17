pub struct Block {
  pub is_last: bool,
  pub length: u32,
  pub data: BlockData,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockData {
  StreamInfo(StreamInfo),
  Padding(u32),
  Application(Application),
  SeekTable(Vec<SeekPoint>),
  VorbisComment(VorbisComment),
  CueSheet(CueSheet),
  Picture(Picture),
  Unknown(Vec<u8>),
}

/// Information regarding the entire audio stream
#[derive(Debug, PartialEq, Eq)]
pub struct StreamInfo {
  pub min_block_size: u16,
  pub max_block_size: u16,
  pub min_frame_size: u32,
  pub max_frame_size: u32,
  pub sample_rate: u32,
  pub channels: u8,
  pub bits_per_sample: u8,
  pub total_samples: u64,
  pub md5_sum: Vec<u8>,
}

/// Data used by third-party applications
#[derive(Debug, PartialEq, Eq)]
pub struct Application {
  pub id: String,
  pub data: Vec<u8>,
}

/// Seek, or skip, to a point within the FLAC file
#[derive(Debug, PartialEq, Eq)]
pub struct SeekPoint {
  pub sample_number: u64,
  pub stream_offset: u64,
  pub frame_samples: u16,
}

/// Stores human-readable name/value pairs
#[derive(Debug, PartialEq, Eq)]
pub struct VorbisComment {
  pub vendor_string: String,
  pub comments: Vec<String>,
}

/// Stores cue information
///
/// Generally for storing information from Compact Disk Digital Audio, but
/// can be used as a cueing mechanism for playback.
#[derive(Debug, PartialEq, Eq)]
pub struct CueSheet {
  pub media_catalog_number: String,
  pub lead_in: u64,
  pub is_cd: bool,
  pub tracks: Vec<CueSheetTrack>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CueSheetTrack {
  pub offset: u64,
  pub number: u8,
  pub isrc: String,
  pub isnt_audio: bool,
  pub is_pre_emphasis: bool,
  pub indices: Vec<CueSheetTrackIndex>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CueSheetTrackIndex {
  pub offset: u64,
  pub number: u8,
}

/// Stores pictures associated with a FLAC file
///
/// More than likely these pictures will be cover art, but you can have more
/// than one within a file, which are distinguished by `PictureType`and it's
/// mime type string.
#[derive(Debug, PartialEq, Eq)]
pub struct Picture {
  pub picture_type: PictureType,
  pub mime_type: String,
  pub description: String,
  pub width: u32,
  pub height: u32,
  pub depth: u32,
  pub colors: u32,
  pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
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
