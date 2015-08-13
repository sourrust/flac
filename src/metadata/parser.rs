use nom::{
  be_u8, be_u16, be_u32, be_u64,
  le_u32,
  IResult,
  ErrorCode, Err,
  Needed,
};

use std::str::from_utf8;

use metadata::{
  Block, BlockData,
  StreamInfo, Application, VorbisComment, CueSheet, Picture,
  SeekPoint, CueSheetTrack, CueSheetTrackIndex, PictureType,
};

use utility::to_u32;

macro_rules! skip_bytes(
  ($input: expr, $length: expr) => (
    {
      match take!($input, $length) {
        IResult::Done(i, bytes)   => {
          let sum = bytes.iter().fold(0, |result, byte| result + byte);

          if sum == 0 {
            IResult::Done(i, bytes)
          } else {
            IResult::Error(Err::Position(ErrorCode::Digit as u32, $input))
          }
        }
        IResult::Error(error)     => IResult::Error(error),
        IResult::Incomplete(need) => IResult::Incomplete(need),
      }
    }
  );
);

named!(pub stream_info <&[u8], BlockData>,
  chain!(
    min_block_size: be_u16 ~
    max_block_size: be_u16 ~
    min_frame_size: map!(take!(3), to_u32) ~
    max_frame_size: map!(take!(3), to_u32) ~
    bytes: take!(8) ~
    md5_sum: take!(16),
    || {
      let sample_rate     = ((bytes[0] as u32) << 12) +
                            ((bytes[1] as u32) << 4)  +
                            ((bytes[2] as u32) >> 4);
      let channels        = (bytes[2] >> 1) & 0b0111;
      let bits_per_sample = ((bytes[2] & 0b01) << 4) +
                            bytes[3] >> 4;
      let total_samples   = (((bytes[3] as u64) & 0x0f) << 32) +
                            ((bytes[4] as u64) << 24) +
                            ((bytes[5] as u64) << 16) +
                            ((bytes[6] as u64) << 8) +
                            (bytes[7] as u64);

      BlockData::StreamInfo(StreamInfo {
        min_block_size: min_block_size,
        max_block_size: max_block_size,
        min_frame_size: min_frame_size,
        max_frame_size: max_frame_size,
        sample_rate: sample_rate,
        channels: channels + 1,
        bits_per_sample: bits_per_sample + 1,
        total_samples: total_samples,
        md5_sum: md5_sum,
      })
    }
  )
);

pub fn padding(input: &[u8], length: u32) -> IResult<&[u8], BlockData> {
  match skip_bytes!(input, length) {
    IResult::Done(i, _)       => IResult::Done(i, BlockData::Padding(0)),
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

pub fn application(input: &[u8], length: u32) -> IResult<&[u8], BlockData> {
  chain!(input,
    id: take_str!(4) ~
    data: take!(length - 4),
    || {
      BlockData::Application(Application {
        id: id,
        data: data,
      })
    }
  )
}

named!(seek_point <&[u8], SeekPoint>,
  chain!(
    sample_number: be_u64 ~
    stream_offset: be_u64 ~
    frame_samples: be_u16,
    || {
      SeekPoint {
        sample_number: sample_number,
        stream_offset: stream_offset,
        frame_samples: frame_samples,
      }
    }
  )
);

fn seek_table(input: &[u8], length: u32) -> IResult<&[u8], BlockData> {
  let seek_count = (length / 18) as usize;

  map!(input, count!(seek_point, seek_count), BlockData::SeekTable)
}

named!(vorbis_comment <&[u8], BlockData>,
  chain!(
    vendor_string_length: le_u32 ~
    vendor_string: take_str!(vendor_string_length)  ~
    number_of_comments: le_u32 ~
    comments: count!(comment_field, number_of_comments as usize),
    || {
      BlockData::VorbisComment(VorbisComment {
        vendor_string: vendor_string,
        comments: comments,
      })
    }
  )
);

named!(comment_field <&[u8], &str>,
  chain!(
    comment_length: le_u32 ~
    comment: take_str!(comment_length),
    || { comment }
  )
);

named!(cue_sheet <&[u8], BlockData>,
  chain!(
    media_catalog_number: take_str!(128) ~
    lead_in: be_u64 ~
    bytes: take!(259) ~ // TODO: last (7 + 258 * 8) bits must be 0
    num_tracks: be_u8 ~
    tracks: count!(cue_sheet_track, num_tracks as usize),
    || {
      let is_cd = ((bytes[0] >> 7) & 0b01) == 1;

      BlockData::CueSheet(CueSheet {
        media_catalog_number: media_catalog_number,
        lead_in: lead_in,
        is_cd: is_cd,
        tracks: tracks,
      })
    }
  )
);

named!(cue_sheet_track <&[u8], CueSheetTrack>,
  chain!(
    offset: be_u64 ~
    number: be_u8 ~
    isrc: take_str!(12) ~
    bytes: take!(14) ~ // TODO: last (6 + 13 * 8) bits must be 0
    num_indices: be_u8 ~
    indices: cond!(
      num_indices != 0,
      count!(cue_sheet_track_index, num_indices as usize)
    ),
    || {
      let isnt_audio      = ((bytes[0] >> 7) & 0b01) == 1;
      let is_pre_emphasis = ((bytes[0] >> 6) & 0b01) == 1;

      CueSheetTrack {
        offset: offset,
        number: number,
        isrc: isrc,
        isnt_audio: isnt_audio,
        is_pre_emphasis: is_pre_emphasis,
        indices: indices.unwrap_or(Vec::new()),
      }
    }
  )
);

named!(cue_sheet_track_index <&[u8], CueSheetTrackIndex>,
  chain!(
    offset: be_u64 ~
    number: be_u8 ~
    skip_bytes!(3),
    || {
      CueSheetTrackIndex {
        offset: offset,
        number: number,
      }
    }
  )
);

named!(picture <&[u8], BlockData>,
  chain!(
    picture_type_num: be_u32 ~
    mime_type_length:  be_u32 ~
    mime_type: take_str!(mime_type_length) ~
    description_length: be_u32 ~
    description: take_str!(description_length) ~
    width: be_u32 ~
    height: be_u32 ~
    depth: be_u32 ~
    colors: be_u32 ~
    data_length: be_u32 ~
    data: take!(data_length),
    || {
      let picture_type = match picture_type_num {
        0  => PictureType::Other,
        1  => PictureType::FileIconStandard,
        2  => PictureType::FileIcon,
        3  => PictureType::FrontCover,
        4  => PictureType::BackCover,
        5  => PictureType::LeafletPage,
        6  => PictureType::Media,
        7  => PictureType::LeadArtist,
        8  => PictureType::Artist,
        9  => PictureType::Conductor,
        10 => PictureType::Band,
        11 => PictureType::Composer,
        12 => PictureType::Lyricist,
        13 => PictureType::RecordingLocation,
        14 => PictureType::DuringRecording,
        15 => PictureType::DuringPerformace,
        16 => PictureType::VideoScreenCapture,
        17 => PictureType::Fish,
        18 => PictureType::Illustration,
        19 => PictureType::BandLogoType,
        20 => PictureType::PublisherLogoType,
        _  => PictureType::Other,
      };

      BlockData::Picture(Picture {
        picture_type: picture_type,
        mime_type: mime_type,
        description: description,
        width: width,
        height: height,
        depth: depth,
        colors: colors,
        data: data,
      })
    }
  )
);

// As of FLAC v1.3.1, there is support for up to 127 different metadata
// `Block`s but actually 7 that are implemented. When the `Block` type isn't
// recognised, this block gets skipped over with this parser.
fn unknown(input: &[u8], length: u32) -> IResult<&[u8], BlockData> {
  map!(input, take!(length), BlockData::Unknown)
}

named!(pub header <&[u8], (bool, u8, u32)>,
  chain!(
    block_byte: be_u8 ~
    length: map!(take!(3), to_u32),
    || {
      let is_last    = (block_byte >> 7) == 1;
      let block_type = block_byte & 0b01111111;

      (is_last, block_type, length)
    }
  )
);

fn block_data(input: &[u8], block_type: u8, length: u32)
              -> IResult<&[u8], BlockData> {
  match block_type {
    0       => stream_info(input),
    1       => padding(input, length),
    2       => application(input, length),
    3       => seek_table(input, length),
    4       => vorbis_comment(input),
    5       => cue_sheet(input),
    6       => picture(input),
    7...126 => unknown(input, length),
    _       => IResult::Error(Err::Position(ErrorCode::Alt as u32, input)),
  }
}

named!(block <&[u8], Block>,
  chain!(
    block_header: header ~
    data: apply!(block_data, block_header.1, block_header.2),
    || {
      Block {
        is_last: block_header.0,
        length: block_header.2,
        data: data
      }
    }
  )
);

/// Parses one or more metadata `Block`s
///
/// The first block should always be `StreamInfo` since that is the only
/// required `Block`. At the moment `many_blocks` doesn't check that
/// requirement.
pub fn many_blocks(input: &[u8]) -> IResult<&[u8], Vec<Block>> {
  let mut is_last   = false;
  let mut blocks    = Vec::new();
  let mut start     = 0;
  let mut remaining = input.len();

  while !is_last {
    match block(&input[start..]) {
      IResult::Done(i, block) => {
        let result_len = i.len();

        if result_len == input[start..].len() {
          break;
        }

        start    += remaining - result_len;
        remaining = result_len;
        is_last   = block.is_last;

        blocks.push(block);
      }
      _                       => break,
    }
  }

  if blocks.len() == 0 {
    IResult::Error(Err::Position(ErrorCode::Many1 as u32, input))
  } else if is_last {
    IResult::Done(&input[start..], blocks)
  } else {
    IResult::Incomplete(Needed::Unknown)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use metadata::{
    BlockData,
    StreamInfo,
  };
  use nom::{
    IResult,
    ErrorCode, Err,
  };

  #[test]
  fn test_header() {
    let inputs = [ [0x80, 0x00, 0x00, 0x22]
                 , [0x01, 0x00, 0x04, 0x00]
                 , [0x84, 0x00, 0x00, 0xf8]
                 ];

    assert!(header(&inputs[0]) == IResult::Done(&[], (true, 0, 34)),
            "Header Test #1");
    assert!(header(&inputs[1]) == IResult::Done(&[], (false, 1, 1024)),
            "Header Test #2");
    assert!(header(&inputs[2]) == IResult::Done(&[], (true, 4, 248)),
            "Header Test #3");
  }

  #[test]
  fn test_stream_info() {
    let input   = [ 0x12, 0x00, 0x12, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x10
                  , 0x01, 0xf4, 0x02, 0x70, 0x00, 0x01, 0x38, 0x80, 0xa0, 0x42
                  , 0x23, 0x7c, 0x54, 0x93, 0xfd, 0xb9, 0x65, 0x6b, 0x94, 0xa8
                  , 0x36, 0x08, 0xd1, 0x1a
                  ];

    let md5_sum = [ 0xa0, 0x42, 0x23, 0x7c, 0x54, 0x93, 0xfd, 0xb9, 0x65, 0x6b
                  , 0x94, 0xa8, 0x36, 0x08, 0xd1, 0x1a
                  ];

    let result = BlockData::StreamInfo(StreamInfo {
      min_block_size: 4608,
      max_block_size: 4608,
      min_frame_size: 14,
      max_frame_size: 16,
      sample_rate: 8000,
      channels: 2,
      bits_per_sample: 8,
      total_samples: 80000,
      md5_sum: &md5_sum,
    });

    assert_eq!(stream_info(&input), IResult::Done(&[][..], result));
  }

  #[test]
  fn test_padding() {
    let input = [ [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                , [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]
                ];

    let result_valid   = IResult::Done(&[][..], BlockData::Padding(0));
    let result_invalid = IResult::Error(Err::Position(
                           ErrorCode::Digit as u32, &input[1][..]));

    assert!(padding(&input[0], 10) == result_valid, "Valid Padding");
    assert!(padding(&input[1], 10) == result_invalid, "Invalid Padding");
  }
}
