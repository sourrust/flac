use nom::{
  be_u8, be_u16,
  IResult,
  ErrorCode, Err,
};

use frame::{
  ChannelAssignment,
  Footer,
};

fn blocking_strategy(input: &[u8]) -> IResult<&[u8], bool> {
  match take!(input, 2) {
    IResult::Done(i, bytes)   => {
      let sync_code = ((bytes[0] as u16) << 6) +
                      (bytes[1] as u16) >> 2;
      let is_valid  = ((bytes[1] >> 1) & 0b01) == 0;

      if sync_code == 0b11111111111110 && is_valid {
        let is_variable_block_size = (bytes[1] & 0b01) == 1;

        IResult::Done(i, is_variable_block_size)
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn block_sample(input: &[u8]) -> IResult<&[u8], (u32, u32)> {
  match take!(input, 1) {
    IResult::Done(i, bytes)   => {
      let sample_rate = bytes[0] & 0x0f;

      if sample_rate != 0x0f {
        let block_size = bytes[0] >> 4;

        IResult::Done(i, (block_size as u32, sample_rate as u32))
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn channel_bits(input: &[u8]) -> IResult<&[u8], (ChannelAssignment, usize)> {
  match take!(input, 1) {
    IResult::Done(i, bytes)   => {
      let channel_assignment = match bytes[0] >> 4 {
        0b0000...0b0111 => ChannelAssignment::Independent,
        0b1000          => ChannelAssignment::LeftSide,
        0b1001          => ChannelAssignment::RightSide,
        0b1010          => ChannelAssignment::MiddleSide,
        _               => ChannelAssignment::Independent,
      };
      let bits_per_sample = (bytes[0] >> 1) & 0b0111;
      let is_valid        = (bytes[0] & 0b01) == 0;

      if is_valid {
        IResult::Done(i, (channel_assignment, bits_per_sample as usize))
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn utf8_size(input: &[u8], is_u64: bool)
             -> IResult<&[u8], Option<(usize, u8)>> {
  map!(input, be_u8, |utf8_header| {
    match utf8_header {
      0b00000000...0b01111111 => Some((0, utf8_header)),
      0b11000000...0b11011111 => Some((1, utf8_header & 0b00011111)),
      0b11100000...0b11101111 => Some((2, utf8_header & 0b00001111)),
      0b11110000...0b11110111 => Some((3, utf8_header & 0b00000111)),
      0b11111000...0b11111011 => Some((4, utf8_header & 0b00000011)),
      0b11111100...0b11111101 => Some((5, utf8_header & 0b00000001)),
      0b11111110              => if is_u64 { Some((6, 0)) } else { None },
      _                       => None,
    }
  })
}

named!(footer <&[u8], Footer>, map!(be_u16, Footer));
