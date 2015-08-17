use nom::{Consumer, FileProducer};
use std::io::{Error, ErrorKind, Result};

use metadata::{
  Block, BlockData,
  StreamInfo, CueSheet, VorbisComment,
  MetaDataConsumer,
};

pub fn get_metadata(filename: &str) -> Result<Vec<Block>> {
  FileProducer::new(filename, 1024).and_then(|mut producer| {
    let mut consumer = MetaDataConsumer::new();

    consumer.run(&mut producer);

    if !consumer.data.is_empty() {
      Ok(consumer.data)
    } else {
      let error_str = "parser: couldn't find any metadata";

      Err(Error::new(ErrorKind::InvalidData, error_str))
    }
  })
}

pub fn get_stream_info(filename: &str) -> Result<StreamInfo> {
  get_metadata(filename).and_then(|blocks| {
    let error_str  = "metadata: couldn't find StreamInfo";
    let mut result = Err(Error::new(ErrorKind::NotFound, error_str));

    for block in blocks {
      if let BlockData::StreamInfo(stream_info) = block.data {
        result = Ok(stream_info);
        break;
      }
    }

    result
  })
}

pub fn get_vorbis_comment(filename: &str) -> Result<VorbisComment> {
  get_metadata(filename).and_then(|blocks| {
    let error_str  = "metadata: couldn't find VorbisComment";
    let mut result = Err(Error::new(ErrorKind::NotFound, error_str));

    for block in blocks {
      if let BlockData::VorbisComment(vorbis_comment) = block.data {
        result = Ok(vorbis_comment);
        break;
      }
    }

    result
  })
}

pub fn get_cue_sheet(filename: &str) -> Result<CueSheet> {
  get_metadata(filename).and_then(|blocks| {
    let error_str  = "metadata: couldn't find CueSheet";
    let mut result = Err(Error::new(ErrorKind::NotFound, error_str));

    for block in blocks {
      if let BlockData::CueSheet(cue_sheet) = block.data {
        result = Ok(cue_sheet);
        break;
      }
    }

    result
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::ErrorKind;

  #[test]
  fn test_get_metadata() {
    let not_found    = get_metadata("non-existent/file.txt");
    let invalid_data = get_metadata("README.md");
    let result       = get_metadata("tests/assets/input-SVAUP.flac");

    assert_eq!(not_found.unwrap_err().kind(), ErrorKind::NotFound);
    assert_eq!(invalid_data.unwrap_err().kind(), ErrorKind::InvalidData);
    assert!(result.is_ok());
  }
}
