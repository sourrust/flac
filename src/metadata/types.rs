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

pub struct StreamInfo<'a> {
  pub min_block_size: u16,
  pub max_block_size: u16,
  pub min_frame_size: u32,
  pub max_frame_size: u32,
  pub sample_rate: u32,
  pub channels: u8,
  pub bits_per_sample: u8,
  pub total_samples: u64,
  pub md5_sum: &'a str,
}

pub struct Application<'a> {
  pub id: &'a str,
  pub data: &'a [u8],
}

pub struct SeekPoint {
  pub sample_number: u64,
  pub stream_offset: u64,
  pub frame_samples: u16,
}

pub struct VorbisComment<'a> {
  pub vendor_string: &'a str,
  pub comments: Vec<&'a str>,
}

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
