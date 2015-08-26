use nom::{IResult, Consumer, ConsumerState};
use metadata::parser::{header, block_data};

use std::collections::HashMap;

/// Data associated with a single metadata block.
#[derive(Debug)]
pub struct Block {
  /// Marks whether the current metadata block is the last.
  pub is_last: bool,
  /// The length, in bytes, of the block being parsed. This does not include
  /// the metadata block header.
  pub length: u32,
  /// Block data containing on of the eight different types of metadata.
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
  pub md5_sum: [u8; 16],
}

/// Data used by third-party applications.
#[derive(Debug, PartialEq, Eq)]
pub struct Application {
  pub id: String,
  pub data: Vec<u8>,
}

/// Seek, or skip, to a point within the FLAC file.
#[derive(Debug, PartialEq, Eq)]
pub struct SeekPoint {
  pub sample_number: u64,
  pub stream_offset: u64,
  pub frame_samples: u16,
}

/// Stores human-readable name/value pairs.
#[derive(Debug, PartialEq, Eq)]
pub struct VorbisComment {
  pub vendor_string: String,
  pub comments: HashMap<String, String>,
}

/// Stores cue information.
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
  pub is_audio: bool,
  pub is_pre_emphasis: bool,
  pub indices: Vec<CueSheetTrackIndex>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CueSheetTrackIndex {
  pub offset: u64,
  pub number: u8,
}

/// Stores pictures associated with the FLAC file.
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
  BandLogo,
  PublisherLogo,
}

enum ParserState {
  FLAC,
  Header,
  MetaData,
}

pub struct MetaDataConsumer {
  state: ParserState,
  is_last: bool,
  block_type: u8,
  length: u32,
  pub data: Vec<Block>
}

impl MetaDataConsumer {
  pub fn new() -> MetaDataConsumer {
    MetaDataConsumer {
      state: ParserState::FLAC,
      is_last: false,
      block_type: 127,
      length: 0,
      data: Vec::new(),
    }
  }
}

impl Consumer for MetaDataConsumer {
  fn consume(&mut self, input: &[u8]) -> ConsumerState {
    match self.state {
      ParserState::FLAC     => {
        match tag!(input, "fLaC") {
          IResult::Done(_, _) => {
            self.state = ParserState::Header;
            ConsumerState::Await(4, 4)
          }
          IResult::Error(_)      => ConsumerState::ConsumerError(0),
          IResult::Incomplete(_) => ConsumerState::Await(0, 4),
        }
      }
      ParserState::Header   => {
        match header(input) {
          IResult::Done(_, data) => {
            let (is_last, block_type, length) = data;

            self.state      = ParserState::MetaData;
            self.is_last    = is_last;
            self.block_type = block_type;
            self.length     = length;

            ConsumerState::Await(4, length as usize)
          }
          IResult::Error(_)      => ConsumerState::ConsumerError(0),
          IResult::Incomplete(_) => ConsumerState::Await(0, 4),
        }
      },
      ParserState::MetaData => {
        let length = self.length as usize;

        match block_data(input, self.block_type, self.length) {
          IResult::Done(_, data) => {
            self.data.push(Block {
              is_last: self.is_last,
              length: self.length,
              data: data,
            });

            if self.is_last {
              ConsumerState::ConsumerDone
            } else {
              self.state = ParserState::Header;

              ConsumerState::Await(length, 4)
            }
          }
          IResult::Error(_)      => ConsumerState::ConsumerError(0),
          IResult::Incomplete(_) => ConsumerState::Await(0, length),
        }
      }
    }
  }

  fn failed(&mut self, error_code: u32) {
    println!("Failed with error code: {}", error_code);
  }

  fn end(&mut self) {}
}
