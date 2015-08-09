use nom::{
  be_u8,
  IResult,
  ErrorCode, Err,
};

use metadata::BlockData;

use utility::to_u32;

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
