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

use metadata::StreamInfo;
use utility::{crc8, crc16, to_u32};

pub fn frame_parser<'a>(input: &'a [u8], stream_info: &StreamInfo)
                        -> IResult<'a, &'a [u8], Frame> {
  let result = chain!(input,
    frame_header: apply!(header, stream_info) ~
    frame_footer: footer,
    || {
      Frame {
        header: frame_header,
        footer: frame_footer,
      }
    }
  );

  match result {
    IResult::Done(i, frame)   => {
      // All frame bytes before the crc-16
      let end         = (input.len() - i.len()) - 2;
      let Footer(crc) = frame.footer;

      if crc16(&input[0..end]) == crc {
        IResult::Done(i, frame)
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn blocking_strategy(input: &[u8]) -> IResult<&[u8], bool> {
  match take!(input, 2) {
    IResult::Done(i, bytes)   => {
      let sync_code = ((bytes[0] as u16) << 6) +
                      ((bytes[1] as u16) >> 2);
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

fn block_sample(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
  match be_u8(input) {
    IResult::Done(i, byte)    => {
      let block_byte  = byte >> 4;
      let sample_byte = byte & 0x0f;
      let is_valid    = block_byte != 0b0000 && sample_byte != 0b1111;

      if is_valid {
        IResult::Done(i, (block_byte, sample_byte))
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

fn channel_bits(input: &[u8]) -> IResult<&[u8], (ChannelAssignment, u8, u8)> {
  match be_u8(input) {
    IResult::Done(i, byte)    => {
      let mut channels       = 2;
      let channel_byte       = byte >> 4;
      let channel_assignment = match channel_byte {
        0b0000...0b0111 => {
          channels = channel_byte + 1;

          ChannelAssignment::Independent
        }
        0b1000          => ChannelAssignment::LeftSide,
        0b1001          => ChannelAssignment::RightSide,
        0b1010          => ChannelAssignment::MiddleSide,
        _               => ChannelAssignment::Independent,
      };
      let size_byte = (byte >> 1) & 0b0111;

      // Checks the validity of:
      //
      // * (4 bits) channel assignment
      // * (3 bits) sample size
      // * (1 bit) last bit
      //
      // All these checks are whether of not they are reserved and should
      // return an error if so.
      let is_valid = channel_byte < 0b1011 &&
                     (size_byte != 0b0011 && size_byte != 0b0111) &&
                     (byte & 0b01) == 0;

      if is_valid {
        IResult::Done(i, (channel_assignment, channels, size_byte))
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

fn number_type(input: &[u8], is_sample: bool,
               (size, value): (usize, u8))
               -> IResult<&[u8], NumberType> {
  let mut result   = value as u64;
  let mut is_error = false;

  match take!(input, size) {
    IResult::Done(i, bytes)   => {
      for i in 0..size {
        let byte = bytes[i] as u64;

        if byte >= 0b10000000 && byte <= 0b10111111 {
          result = (result << 6) + (byte & 0b00111111);
        } else {
          is_error = true;
          break;
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

fn secondary_block_size(input: &[u8], block_byte: u8)
                        -> IResult<&[u8], Option<u32>> {
  match block_byte {
    0b0110 => opt!(input, map!(take!(1), to_u32)),
    0b0111 => opt!(input, map!(take!(2), to_u32)),
    _      => IResult::Done(input, None)
  }
}

fn secondary_sample_rate(input: &[u8], sample_byte: u8)
                        -> IResult<&[u8], Option<u32>> {
  match sample_byte {
    0b1100          => opt!(input, map!(take!(1), to_u32)),
    0b1101 | 0b1110 => opt!(input, map!(take!(2), to_u32)),
    _               => IResult::Done(input, None)
  }
}

pub fn header<'a>(input: &'a [u8], stream_info: &StreamInfo)
                  -> IResult<'a, &'a [u8], Header> {
  let result = chain!(input,
    is_variable_block_size: blocking_strategy ~
    tuple0: block_sample ~
    tuple1: channel_bits ~
    number_opt: apply!(utf8_size, is_variable_block_size) ~
    number_length: expr_opt!(number_opt) ~
    number: apply!(number_type, is_variable_block_size, number_length) ~
    alt_block_size: apply!(secondary_block_size, tuple0.0) ~
    alt_sample_rate: apply!(secondary_sample_rate, tuple0.1) ~
    crc: be_u8,
    || {
      let (block_byte, sample_byte)       = tuple0;
      let (channel_assignment, size_byte) = tuple1;

      let block_size = match block_byte {
        0b0001          => 192,
        0b0010...0b0101 => 576 * 2_u32.pow(block_byte as u32 - 2),
        0b0110 | 0b0111 => alt_block_size.unwrap() + 1,
        0b1000...0b1111 => 256 * 2_u32.pow(block_byte as u32 - 8),
        _               => unreachable!(),
      };

      let sample_rate = match sample_byte {
        0b0000 => stream_info.sample_rate,
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
        _      => unreachable!(),
      };

      let bits_per_sample = match size_byte {
        0b0000 => stream_info.bits_per_sample as usize,
        0b0001 => 8,
        0b0010 => 12,
        0b0100 => 16,
        0b0101 => 20,
        0b0110 => 24,
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
  );

  match result {
    IResult::Done(i, frame_header) => {
      // All header bytes before the crc-8
      let end = (input.len() - i.len()) - 1;

      if crc8(&input[0..end]) == frame_header.crc {
        IResult::Done(i, frame_header)
      } else {
        IResult::Error(Err::Position(ErrorCode::Digit as u32, input))
      }
    }
    IResult::Error(error)          => IResult::Error(error),
    IResult::Incomplete(need)      => IResult::Incomplete(need),
  }
}

named!(pub footer <&[u8], Footer>, map!(be_u16, Footer));

#[cfg(test)]
mod tests {
  use super::*;
  use frame::{
    Header, Footer,
    ChannelAssignment, NumberType,
  };
  use metadata::StreamInfo;
  use nom::IResult;

  #[test]
  fn test_header() {
    let inputs   = [ &b"\xff\xf8\x53\x1c\xf0\x90\x80\x80\x2e"[..]
                   , &b"\xff\xf9\x7c\xa0\xfe\xbf\xbf\xbf\xbf\xbf\xbc\x01\xff\
                        \x01\x88"[..]
                   , &b"\xff\xf8\xc8\x72\x40\x19"[..]
                   ];
    let mut info = StreamInfo::new();
    let results  = [ IResult::Done(&[][..], Header {
                      block_size: 4608,
                      sample_rate: 192000,
                      channel_assignment: ChannelAssignment::Independent,
                      bits_per_sample: 24,
                      number: NumberType::Frame(65536),
                      crc: 0x2e,
                    })
                  , IResult::Done(&[][..], Header {
                      block_size: 512,
                      sample_rate: 1000,
                      channel_assignment: ChannelAssignment::MiddleSide,
                      bits_per_sample: 16,
                      number: NumberType::Sample(68719476732),
                      crc: 0x88,
                    })
                  , IResult::Done(&[][..], Header {
                      block_size: 4096,
                      sample_rate: 32000,
                      channel_assignment: ChannelAssignment::Independent,
                      bits_per_sample: 8,
                      number: NumberType::Frame(64),
                      crc: 0x19,
                    })
                  ];

    info.bits_per_sample = 16;

    assert_eq!(header(inputs[0], &info), results[0]);
    assert_eq!(header(inputs[1], &info), results[1]);
    assert_eq!(header(inputs[2], &info), results[2]);
  }

  #[test]
  fn test_footer() {
    let input  = b"\x03\xe8";
    let result = IResult::Done(&[][..], Footer(0x03e8));

    assert_eq!(footer(input), result);
  }
}
