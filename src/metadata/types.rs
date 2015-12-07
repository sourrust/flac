use nom::{
  Consumer, ConsumerState,
  ErrorKind,
  HexDisplay,
  Input, IResult,
  Move, Needed,
};
use metadata::parser::{header, block_data};

use std::collections::HashMap;

/// Data associated with a single metadata block.
#[derive(Debug)]
pub struct Metadata {
  /// Marks whether the current metadata block is the last.
  pub is_last: bool,
  /// The length, in bytes, of the block being parsed. This does not include
  /// the metadata block header.
  pub length: u32,
  /// Block data containing one of the eight different types of metadata.
  pub data: Data,
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
}

/// Data used by third-party applications.
#[derive(Debug, PartialEq, Eq)]
pub struct Application {
  /// Registered application ID.
  pub id: String,
  /// Data used by the third-party application.
  pub data: Vec<u8>,
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
  DuringPerformace,
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

enum ParserState {
  Marker,
  Header,
  Block((bool, u8, u32)),
}

pub struct MetaDataConsumer {
  state: ParserState,
  consumer_state: ConsumerState<(), ErrorKind, Move>,
  pub data: Vec<Metadata>,
}

impl MetaDataConsumer {
  pub fn new() -> MetaDataConsumer {
    let consumed = Move::Consume(0);

    MetaDataConsumer {
      state: ParserState::Marker,
      consumer_state: ConsumerState::Continue(consumed),
      data: Vec::new(),
    }
  }

  fn handle_marker<'a>(&mut self, input: &'a [u8]) -> IResult<&'a [u8], ()> {
    let kind = ErrorKind::Custom(0);

    match tag!(input, "fLaC") {
      IResult::Done(i, _)    => {
        self.state = ParserState::Header;

        IResult::Error(Err::Position(kind, i))
      }
      IResult::Error(_)      => IResult::Error(Err::Code(kind)),
      IResult::Incomplete(n) => IResult::Incomplete(n),
    }
  }

  fn handle_header<'a>(&mut self, input: &'a [u8]) -> IResult<&'a [u8], ()> {
    match header(input) {
      IResult::Done(i, data) => {
        let offset   = input.offset(i);
        let consumed = Move::Consume(offset);

        self.state          = ParserState::Block(data);
        self.consumer_state = ConsumerState::Continue(consumed);
      }
      IResult::Error(_)      => {
        let kind = ErrorKind::Custom(1);

        self.consumer_state = ConsumerState::Error(kind);
      }
      IResult::Incomplete(_) => {
        let needed = Move::Await(Needed::Size(4));

        self.consumer_state = ConsumerState::Continue(needed);
      }
    }
  }

  fn handle_block<'a>(&mut self, input: &'a [u8], header: (bool, u8, u32))
                      -> IResult<&'a [u8], ()> {
    let (is_last, block_type, length) = header;

    match block_data(input, block_type, length) {
      IResult::Done(i, data) => {
        let offset   = input.offset(i);
        let consumed = Move::Consume(offset);

        self.data.push(Metadata {
          is_last: is_last,
          length: length,
          data: data,
        });

        if is_last {
          self.consumer_state = ConsumerState::Done(consumed, ());
        } else {
          self.state          = ParserState::Header;
          self.consumer_state = ConsumerState::Continue(consumed);
        }
      }
      IResult::Error(_)      => {
        let kind = ErrorKind::Custom(2);

        self.consumer_state = ConsumerState::Error(kind);
      }
      IResult::Incomplete(_) => {
        let needed = Move::Await(Needed::Size(length as usize));

        self.consumer_state = ConsumerState::Continue(needed);
      }
    }
  }
}

impl<'a> Consumer<&'a [u8], (), ErrorKind, Move> for MetaDataConsumer {
  fn state(&self) -> &ConsumerState<(), ErrorKind, Move> {
    &self.consumer_state
  }

  fn handle(&mut self, input: Input<&'a [u8]>)
            -> &ConsumerState<(), ErrorKind, Move> {
    match input {
      Input::Element(i) | Input::Eof(Some(i)) => {
        match self.state {
          ParserState::Marker      => self.handle_marker(i),
          ParserState::Header      => self.handle_header(i),
          ParserState::Block(data) => self.handle_block(i, data),
        }
      }
      Input::Empty | Input::Eof(None)         => {
        let kind = match self.state {
          ParserState::Marker   => ErrorKind::Custom(0),
          ParserState::Header   => ErrorKind::Custom(1),
          ParserState::Block(_) => ErrorKind::Custom(2),
        };

        self.consumer_state = ConsumerState::Error(kind);
      }
    }

    &self.consumer_state
  }
}
