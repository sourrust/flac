mod types;
mod parser;
mod metadata;

pub use self::types::{
  Block, BlockData,
  StreamInfo, Application, VorbisComment, CueSheet, Picture,
  SeekPoint, CueSheetTrack, CueSheetTrackIndex, PictureType,
  MetaDataConsumer,
};

pub use self::parser::metadata_parser;

pub use self::metadata::{
  get_stream_info, get_vorbis_comment, get_cue_sheet, get_picture,
};
