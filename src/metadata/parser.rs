use nom::{
  be_u8, be_u16,
  IResult,
  ErrorCode, Err,
};

use std::str::from_utf8;

use metadata::{
  BlockData,
  StreamInfo,
};

use utility::to_u32;

named!(stream_info <&[u8], BlockData>,
  chain!(
    min_block_size: be_u16 ~
    max_block_size: be_u16 ~
    min_frame_size: map!(take!(3), to_u32) ~
    max_frame_size: map!(take!(3), to_u32) ~
    bytes: take!(8) ~
    md5_sum: take_str!(16),
    || {
      let sample_rate     = ((bytes[0] as u32) << 12) +
                            ((bytes[1] as u32) << 4)  +
                            (bytes[2] as u32) >> 4;
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

named!(header <&[u8], (u8, bool, u32)>,
  chain!(
    block_byte: be_u8 ~
    length: map!(take!(3), to_u32),
    || {
      let is_last    = (block_byte >> 7) == 1;
      let block_type = block_byte & 0b01111111;

      (block_type, is_last, length)
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
