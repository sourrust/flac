use nom::{Consumer, FileProducer};
use std::io::{Error, ErrorKind, Result};
use std::u32;

use metadata::{
  Block, BlockData,
  StreamInfo, CueSheet, VorbisComment, Picture,
  PictureType,
  MetaDataConsumer,
};

macro_rules! optional_eq (
  ($compare: expr, $option: expr) => (
    $option.map_or(true, |compare| $compare == compare);
  );
);

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

pub fn get_picture(filename: &str,
                   picture_type: Option<PictureType>,
                   mime_type: Option<&str>,
                   description: Option<&str>,
                   max_width: Option<u32>,
                   max_height: Option<u32>,
                   max_depth: Option<u32>,
                   max_colors: Option<u32>)
                   -> Result<Picture> {
  get_metadata(filename).and_then(|blocks| {
    let error_str  = "metadata: couldn't find any Picture";
    let mut result = Err(Error::new(ErrorKind::NotFound, error_str));

    let mut max_area_seen  = 0;
    let mut max_depth_seen = 0;

    let max_value      = u32::max_value();
    let max_width_num  = max_width.unwrap_or(max_value);
    let max_height_num = max_height.unwrap_or(max_value);
    let max_depth_num  = max_depth.unwrap_or(max_value);
    let max_colors_num = max_colors.unwrap_or(max_value);

    for block in blocks {
      if let BlockData::Picture(picture) = block.data {
        let area = (picture.width as u64) * (picture.height as u64);

        if optional_eq!(picture.picture_type, picture_type) &&
           optional_eq!(picture.mime_type, mime_type) &&
           optional_eq!(picture.description, description) &&
           picture.width <= max_width_num &&
           picture.height <= max_height_num &&
           picture.depth <= max_depth_num &&
           picture.colors <= max_colors_num &&
           (area > max_area_seen || (area == max_area_seen &&
                                     picture.depth > max_depth_seen)) {
          max_area_seen  = area;
          max_depth_seen = picture.depth;
          result         = Ok(picture);
        }
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
