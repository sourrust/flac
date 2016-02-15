use nom::{
  be_u8, be_u16, be_u32, be_u64,
  le_u32,
  IResult, Needed,
  ErrorKind, Err,
};

use std::collections::HashMap;

use metadata::{
  self, Metadata,
  StreamInfo, Application, VorbisComment, CueSheet, Picture,
  SeekPoint, CueSheetTrack, CueSheetTrackIndex, PictureType,
};

use utility::to_u32;

/// Parse a metadata block.
pub fn metadata_parser(input: &[u8]) -> IResult<&[u8], Metadata> {
  chain!(input,
    block_header: header ~
    data: apply!(block_data, block_header.1, block_header.2),
    || { Metadata::new(block_header.0, block_header.2, data) }
  )
}

named!(pub stream_info <&[u8], metadata::Data>,
  chain!(
    min_block_size: be_u16 ~
    max_block_size: be_u16 ~
    min_frame_size: map!(take!(3), to_u32) ~
    max_frame_size: map!(take!(3), to_u32) ~
    bytes: take!(8) ~
    md5_sum: count_fixed!(u8, be_u8, 16),
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

      metadata::Data::StreamInfo(StreamInfo {
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

pub fn padding(input: &[u8], length: u32) -> IResult<&[u8], metadata::Data> {
  map!(input, skip_bytes!(length), |_| metadata::Data::Padding(0))
}

pub fn application(input: &[u8], length: u32)
                   -> IResult<&[u8], metadata::Data> {
  chain!(input,
    id: take_str!(4) ~
    data: take!(length - 4),
    || {
      metadata::Data::Application(Application {
        id: id.to_owned(),
        data: data.to_owned(),
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

pub fn seek_table(input: &[u8], length: u32)
                  -> IResult<&[u8], metadata::Data> {
  let seek_count = (length / 18) as usize;

  map!(input, count!(seek_point, seek_count), metadata::Data::SeekTable)
}

named!(pub vorbis_comment <&[u8], metadata::Data>,
  chain!(
    vendor_string_length: le_u32 ~
    vendor_string: take_str!(vendor_string_length)  ~
    number_of_comments: le_u32 ~
    comment_lines: count!(comment_field, number_of_comments as usize),
    || {
      let mut comments = HashMap::with_capacity(comment_lines.len());

      for line in comment_lines {
        let comment: Vec<&str> = line.splitn(2, '=').collect();

        comments.insert(comment[0].to_owned(), comment[1].to_owned());
      }

      metadata::Data::VorbisComment(VorbisComment {
        vendor_string: vendor_string.to_owned(),
        comments: comments,
      })
    }
  )
);

named!(comment_field <&[u8], String>,
  chain!(
    comment_length: le_u32 ~
    comment: take_str!(comment_length),
    || { comment.to_owned() }
  )
);

named!(pub cue_sheet <&[u8], metadata::Data>,
  chain!(
    media_catalog_number: take_str!(128) ~
    lead_in: be_u64 ~
    // First bit is a flag to check if the cue sheet information is from a
    // Compact Disc. Rest of the bits should be all zeros.
    bytes: skip_bytes!(259, 1) ~
    num_tracks: be_u8 ~
    tracks: count!(cue_sheet_track, num_tracks as usize),
    || {
      let is_cd = (bytes[0] >> 7) == 1;

      metadata::Data::CueSheet(CueSheet {
        media_catalog_number: media_catalog_number.to_owned(),
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
    // First two bits are flags for checking if the track information is
    // apart of some audio and if the audio has been recorded with
    // pre-emphasis.
    bytes: skip_bytes!(14, 2) ~
    num_indices: be_u8 ~
    indices: count!(cue_sheet_track_index, num_indices as usize),
    || {
      let is_audio        = (bytes[0] >> 7) == 0;
      let is_pre_emphasis = ((bytes[0] >> 6) & 0b01) == 1;

      CueSheetTrack {
        offset: offset,
        number: number,
        isrc: isrc.to_owned(),
        is_audio: is_audio,
        is_pre_emphasis: is_pre_emphasis,
        indices: indices,
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

named!(pub picture <&[u8], metadata::Data>,
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
        15 => PictureType::DuringPerformance,
        16 => PictureType::VideoScreenCapture,
        17 => PictureType::Fish,
        18 => PictureType::Illustration,
        19 => PictureType::BandLogo,
        20 => PictureType::PublisherLogo,
        _  => PictureType::Other,
      };

      metadata::Data::Picture(Picture {
        picture_type: picture_type,
        mime_type: mime_type.to_owned(),
        description: description.to_owned(),
        width: width,
        height: height,
        depth: depth,
        colors: colors,
        data: data.to_owned(),
      })
    }
  )
);

// As of FLAC v1.3.1, there is support for up to 127 different metadata
// `Metadata`s but actually 7 that are implemented. When the `Metadata` type
// isn't recognised, this block gets skipped over with this parser.
pub fn unknown(input: &[u8], length: u32) -> IResult<&[u8], metadata::Data> {
  map!(input, take!(length), |data: &[u8]|
    metadata::Data::Unknown(data.to_owned()))
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

pub fn block_data(input: &[u8], block_type: u8, length: u32)
                  -> IResult<&[u8], metadata::Data> {
  let len = length as usize;

  if len > input.len() {
    let needed = Needed::Size(len);

    return IResult::Incomplete(needed);
  }

  match block_type {
    0       => stream_info(input),
    1       => padding(input, length),
    2       => application(input, length),
    3       => seek_table(input, length),
    4       => vorbis_comment(input),
    5       => cue_sheet(input),
    6       => picture(input),
    7...126 => unknown(input, length),
    _       => IResult::Error(Err::Position(ErrorKind::Alt, input)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use metadata;
  use metadata::{
    StreamInfo, Application, VorbisComment, CueSheet, Picture,
    SeekPoint, CueSheetTrack, CueSheetTrackIndex, PictureType,
  };
  use nom::{
    IResult,
    ErrorKind, Err,
  };

  use std::collections::HashMap;

  #[test]
  fn test_header() {
    let inputs = [b"\x80\0\0\x22", b"\x01\0\x04\0", b"\x84\0\0\xf8"];
    let slice  = &[][..];

    assert_eq!(header(inputs[0]), IResult::Done(slice, (true, 0, 34)));
    assert_eq!(header(inputs[1]), IResult::Done(slice, (false, 1, 1024)));
    assert_eq!(header(inputs[2]), IResult::Done(slice, (true, 4, 248)));
  }

  #[test]
  fn test_stream_info() {
    let input  = b"\x12\0\x12\0\0\0\x0e\0\0\x10\x01\xf4\x02\x70\0\x01\x38\x80\
                   \xa0\x42\x23\x7c\x54\x93\xfd\xb9\x65\x6b\x94\xa8\x36\x08\
                   \xd1\x1a";
    let result = metadata::Data::StreamInfo(StreamInfo {
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
    });

    assert_eq!(stream_info(input), IResult::Done(&[][..], result));
  }

  #[test]
  fn test_padding() {
    let inputs = [b"\0\0\0\0\0\0\0\0\0\0", b"\0\0\0\0\x01\0\0\0\0\0"];

    let result_valid   = IResult::Done(&[][..], metadata::Data::Padding(0));
    let result_invalid = IResult::Error(Err::Position(
                           ErrorKind::Digit, &inputs[1][..]));

    assert_eq!(padding(inputs[0], 10), result_valid);
    assert_eq!(padding(inputs[1], 10), result_invalid);
  }

  #[test]
  fn test_application() {
    let inputs  = [&b"fake"[..], &b"rifffake data"[..]];
    let results = [
      IResult::Done(&[][..], metadata::Data::Application(Application {
        id: "fake".to_owned(),
        data: vec![],
      })),
      IResult::Done(&[][..], metadata::Data::Application(Application {
        id: "riff".to_owned(),
        data: inputs[1][4..].to_owned(),
      }))
    ];

    assert_eq!(application(inputs[0], 4), results[0]);
    assert_eq!(application(inputs[1], 13), results[1]);
  }

  #[test]
  fn test_seek_table() {
    let input  = b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x12\0\0\0\0\0\0\0\x12\0\0\
                   \0\0\0\0\0\0\x0e\x04\xf8\xff\xff\xff\xff\xff\xff\xff\xff\0\
                   \0\0\0\0\0\0\0\0\0\xff\xff\xff\xff\xff\xff\xff\xff\0\0\0\0\
                   \0\0\0\0\0\0\xff\xff\xff\xff\xff\xff\xff\xff\0\0\0\0\0\0\0\
                   \0\0\0";
    let result = IResult::Done(&[][..], metadata::Data::SeekTable(vec![
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
    ]));

    assert_eq!(seek_table(input, 5 * 18), result);
  }

  #[test]
  fn test_vorbis_comment() {
    let input = b"\x20\0\0\0reference libFLAC 1.1.3 20060805\x06\0\0\0\
                  \x20\0\0\0REPLAYGAIN_TRACK_PEAK=0.99996948\
                  \x1e\0\0\0REPLAYGAIN_TRACK_GAIN=-7.89 dB\
                  \x20\0\0\0REPLAYGAIN_ALBUM_PEAK=0.99996948\
                  \x1e\0\0\0REPLAYGAIN_ALBUM_GAIN=-7.89 dB\
                  \x08\0\0\0artist=1\x07\0\0\0title=2";

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

    let result = IResult::Done(&[][..],
      metadata::Data::VorbisComment(VorbisComment{
        vendor_string: "reference libFLAC 1.1.3 20060805".to_owned(),
        comments: comments,
      }));

    assert_eq!(vorbis_comment(input), result);
  }

  #[test]
  fn test_cue_sheet() {
    let input  = b"1234567890123\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\x01\x58\x88\x80\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x03\0\0\0\0\0\0\0\0\x01\0\
                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x02\0\0\
                   \0\0\0\0\0\0\x01\0\0\0\0\0\0\0\0\0\x02\x4c\x02\0\0\0\0\0\0\
                   \0\0\0\x0b\x7c\x02\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\x01\0\0\0\0\0\0\0\0\x01\0\0\0\0\0\0\0\0\0\x16\
                   \xf8\xaa\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0";
    let result = IResult::Done(&[][..],
      metadata::Data::CueSheet(CueSheet {
        media_catalog_number: "1234567890123\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                               \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                               \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                               \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                               \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                               \0\0\0\0\0\0\0".to_owned(),
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
      }));

    assert_eq!(cue_sheet(input), result);
  }

  #[test]
  fn test_picture() {
    let input  = b"\0\0\0\0\0\0\0\x09image/png\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\
                   \0\0\0\0\0\0\0\0\0";
    let result = IResult::Done(&[][..],
      metadata::Data::Picture(Picture {
        picture_type: PictureType::Other,
        mime_type: "image/png".to_owned(),
        description: String::new(),
        width: 0,
        height: 0,
        depth: 0,
        colors: 0,
        data: vec![],
      }));

    assert_eq!(picture(input), result);
  }

  #[test]
  fn test_unknown() {
    let input  = b"random data that won't really be parsed anyway.";
    let result = IResult::Done(&[][..],
                   metadata::Data::Unknown(input[..].to_owned()));

    assert_eq!(unknown(input, 47), result);
  }
}
