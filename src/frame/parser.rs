use nom::{
  be_u8, be_u16,
  IResult,
  ErrorCode, Err,
};

use frame::{
  ChannelAssignment, NumberType,
  Frame,
  Header, Footer,
};

use utility::to_u32;

pub fn frame_parser(input: &[u8], channels: u8) -> IResult<&[u8], Frame> {
  chain!(input,
    frame_header: header ~
    frame_footer: footer,
    || {
      Frame {
        header: frame_header,
        footer: frame_footer,
      }
    }
  )
}

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

fn sample_or_frame_number(input: &[u8], is_sample: bool,
                          (size, value): (usize, u8))
                          -> IResult<&[u8], NumberType> {
  let mut result   = value as u64;
  let mut is_error = false;

  match take!(input, size) {
    IResult::Done(i, bytes)   => {
      for i in (0..size) {
        let byte = bytes[i] as u64;

        match byte {
          0b10000000...10111111 => {
            result = (result << 6) + (byte & 0b00111111);
          }
          _                     => {
            is_error = true;
            break;
          }
        }
      }

      if is_error {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input))
      } else if is_sample {
        IResult::Done(i, NumberType::Sample(result))
      } else {
        IResult::Done(i, NumberType::Frame(result as u32))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn secondary_block_size(input: &[u8], block_byte: u32)
                        -> IResult<&[u8], Option<u32>> {
  match block_byte {
    0b0110 => opt!(input, map!(take!(1), to_u32)),
    0b0111 => opt!(input, map!(take!(2), to_u32)),
    _      => IResult::Done(input, None)
  }
}

fn secondary_sample_rate(input: &[u8], sample_byte: u32)
                        -> IResult<&[u8], Option<u32>> {
  match sample_byte {
    0b1100 => opt!(input, map!(take!(1), to_u32)),
    0b1101 => opt!(input, map!(take!(2), to_u32)),
    0b1110 => opt!(input, map!(take!(2), to_u32)),
    _      => IResult::Done(input, None)
  }
}

named!(header <&[u8], Header>,
  chain!(
    is_variable_block_size: blocking_strategy ~
    tuple0: block_sample ~
    tuple1: channel_bits ~
    number_opt: apply!(utf8_size, is_variable_block_size) ~
    number_length: expr_opt!(number_opt) ~
    number: apply!(sample_or_frame_number, is_variable_block_size,
                   number_length) ~
    alt_block_size: apply!(secondary_block_size, tuple0.0) ~
    alt_sample_rate: apply!(secondary_sample_rate, tuple0.1) ~
    crc: be_u8,
    || {
      let (block_byte, sample_byte)             = tuple0;
      let (channel_assignment, bits_per_sample) = tuple1;

      let block_size = match block_byte {
        0b0000          => 0,
        0b0001          => 192,
        0b0010...0b0101 => 576 * 2_u32.pow(tuple0.0 - 2),
        0b0110 | 0b0111 => alt_block_size.unwrap() + 1,
        0b1000...0b1111 => 256 * 2_u32.pow(tuple0.0 - 8),
        _               => unreachable!(),
      };

      let sample_rate = match sample_byte {
        0b0000 => 0,
        0b0001 => 88200,
        0b0010 => 176400,
        0b0011 => 192000,
        0b0100 => 8000,
        0b0101 => 16000,
        0b0110 => 22050,
        0b0111 => 24000,
        0b1000 => 32000,
        0b1001 => 44100,
        0b1010 => 48000,
        0b1011 => 96000,
        0b1100 => alt_sample_rate.unwrap() * 1000,
        0b1101 => alt_sample_rate.unwrap(),
        0b1110 => alt_sample_rate.unwrap() * 10,
        0b1111 => 0,
        _      => unreachable!(),
      };

      Header {
        block_size: block_size,
        sample_rate: sample_rate,
        channel_assignment: channel_assignment,
        bits_per_sample: bits_per_sample,
        number: number,
        crc: crc,
      }
    }
  )
);

named!(footer <&[u8], Footer>, map!(be_u16, Footer));
