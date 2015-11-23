use nom::{
  be_u8, be_u16,
  IResult,
  ErrorKind, Err,
};

use std::mem;

use frame::{
  MAX_CHANNELS,
  ChannelAssignment, NumberType,
  Frame,
  Header, Footer,
};
use subframe::{subframe_parser, Subframe};

use metadata::StreamInfo;
use utility::{crc8, crc16, to_u32};

/// Parses an audio frame
pub fn frame_parser<'a>(input: &'a [u8], stream_info: &StreamInfo)
                        -> IResult<&'a [u8], Frame> {
  // Unsafe way to initialize subframe data, but I would rather do this
  // than have `Subframe` derive `Copy` to do something like:
  //
  // ```
  // let mut subframe = [Subframe {
  //                       data: subframe::Constant(0),
  //                       wasted_bits: 0,
  //                     }; MAX_CHANNELS];
  // ```
  let mut subframes: [Subframe; MAX_CHANNELS] = unsafe { mem::zeroed() };
  let mut channel = 0;

  let result = chain!(input,
    frame_header: apply!(header, stream_info) ~
    bits!(
      count_slice!(
        apply!(subframe_parser, &mut channel, &frame_header),
        &mut subframes[0..(frame_header.channels as usize)]
      )
    ) ~
    frame_footer: footer,
    || {
      Frame {
        header: frame_header,
        subframes: subframes,
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
        IResult::Error(Err::Position(ErrorKind::Digit, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

// Parses the first two bytes of a frame header. There are two things that
// need to be valid inside these two bytes, the 14 bit sync code and the
// following bit must be zero. The last bit is whether or not the block size
// is fixed or varied.
pub fn blocking_strategy(input: &[u8]) -> IResult<&[u8], bool> {
  match take!(input, 2) {
    IResult::Done(i, bytes)   => {
      let sync_code = ((bytes[0] as u16) << 6) +
                      ((bytes[1] as u16) >> 2);
      let is_valid  = sync_code == 0b11111111111110 &&
                      ((bytes[1] >> 1) & 0b01) == 0;

      if is_valid {
        let is_variable_block_size = (bytes[1] & 0b01) == 1;

        IResult::Done(i, is_variable_block_size)
      } else {
        IResult::Error(Err::Position(ErrorKind::Digit, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

// Parses the third byte of a frame header. There are two four bit values
// that can't be a certain value. For block size bits, it can't be zero
// because that value is reserved. And sample rate bits can't be 0b1111 to
// prevent sync code fooling.
pub fn block_sample(input: &[u8]) -> IResult<&[u8], (u8, u8)> {
  match be_u8(input) {
    IResult::Done(i, byte)    => {
      let block_byte  = byte >> 4;
      let sample_byte = byte & 0b1111;
      let is_valid    = block_byte != 0b0000 && sample_byte != 0b1111;

      if is_valid {
        IResult::Done(i, (block_byte, sample_byte))
      } else {
        IResult::Error(Err::Position(ErrorKind::Digit, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

// Parses the fourth byte of a frame header. There are three values that
// need validation within the byte. First is the channel assignment bits
// which can't be more than 0b1010. Second is the sample size bits that have
// two values it can not equal, 0b011 and 0b111. Last is the final bit must
// be a zero.
pub fn channel_bits(input: &[u8])
                    -> IResult<&[u8], (ChannelAssignment, u8, u8)> {
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
        0b1010          => ChannelAssignment::MidpointSide,
        _               => ChannelAssignment::Independent,
      };
      let size_byte = (byte >> 1) & 0b0111;
      let is_valid  = channel_byte < 0b1011 &&
                      (size_byte != 0b0011 && size_byte != 0b0111) &&
                      (byte & 0b01) == 0;

      if is_valid {
        IResult::Done(i, (channel_assignment, channels, size_byte))
      } else {
        IResult::Error(Err::Position(ErrorKind::Digit, input))
      }
    }
    IResult::Error(error)     => IResult::Error(error),
    IResult::Incomplete(need) => IResult::Incomplete(need),
  }
}

// Similar to the way UTF-8 strings are parsed, only extends to UCS-2 when
// it is a larger sized header. When we hit the branch that check for the
// boolean `is_u64` is when the UCS-2 extension happens and all other
// branches are valid UTF-8 headers.
pub fn utf8_header(input: &[u8], is_u64: bool)
                   -> IResult<&[u8], Option<(usize, u8)>> {
  map!(input, be_u8, |byte| {
    match byte {
      0b00000000...0b01111111 => Some((0, byte)),
      0b11000000...0b11011111 => Some((1, byte & 0b00011111)),
      0b11100000...0b11101111 => Some((2, byte & 0b00001111)),
      0b11110000...0b11110111 => Some((3, byte & 0b00000111)),
      0b11111000...0b11111011 => Some((4, byte & 0b00000011)),
      0b11111100...0b11111101 => Some((5, byte & 0b00000001)),
      0b11111110              => if is_u64 { Some((6, 0)) } else { None },
      _                       => None,
    }
  })
}

// Calculates the value of UTF-8 the next bytes after it's header. The
// header holds both the size and part of this parsers returning value.
pub fn number_type(input: &[u8], is_sample: bool,
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
        IResult::Error(Err::Position(ErrorKind::Digit, input))
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

pub fn secondary_block_size(input: &[u8], block_byte: u8)
                            -> IResult<&[u8], Option<u32>> {
  match block_byte {
    0b0110 => opt!(input, map!(take!(1), to_u32)),
    0b0111 => opt!(input, map!(take!(2), to_u32)),
    _      => IResult::Done(input, None)
  }
}

pub fn secondary_sample_rate(input: &[u8], sample_byte: u8)
                             -> IResult<&[u8], Option<u32>> {
  match sample_byte {
    0b1100          => opt!(input, map!(take!(1), to_u32)),
    0b1101 | 0b1110 => opt!(input, map!(take!(2), to_u32)),
    _               => IResult::Done(input, None)
  }
}

pub fn header<'a>(input: &'a [u8], stream_info: &StreamInfo)
                  -> IResult<&'a [u8], Header> {
  let result = chain!(input,
    is_variable_block_size: blocking_strategy ~
    tuple0: block_sample ~
    tuple1: channel_bits ~
    utf8_header_opt: apply!(utf8_header, is_variable_block_size) ~
    utf8_header_val: expr_opt!(utf8_header_opt) ~
    number: apply!(number_type, is_variable_block_size, utf8_header_val) ~
    alt_block_size: apply!(secondary_block_size, tuple0.0) ~
    alt_sample_rate: apply!(secondary_sample_rate, tuple0.1) ~
    crc: be_u8,
    || {
      let (block_byte, sample_byte)                 = tuple0;
      let (channel_assignment, channels, size_byte) = tuple1;

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
        channels: channels,
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
        IResult::Error(Err::Position(ErrorKind::Digit, input))
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
  use nom::{IResult, Err, ErrorKind};

  fn error<O>(input: &[u8]) -> IResult<&[u8], O> {
    IResult::Error(Err::Position(ErrorKind::Digit, input))
  }

  #[test]
  fn test_blocking_strategy() {
    let inputs = [b"\xff\xf8", b"\xff\xf9", b"\xfe\xf8", b"\xff\xfa"];
    let slice  = &[][..];

    assert_eq!(blocking_strategy(inputs[0]), IResult::Done(slice, false));
    assert_eq!(blocking_strategy(inputs[1]), IResult::Done(slice, true));
    assert_eq!(blocking_strategy(inputs[2]), error(inputs[2]));
    assert_eq!(blocking_strategy(inputs[3]), error(inputs[3]));
  }

  #[test]
  fn test_block_sample() {
    let inputs = [b"\xf9", b"\x1a", b"\x0b", b"\x4f"];
    let slice  = &[][..];

    assert_eq!(block_sample(inputs[0]), IResult::Done(slice, (0x0f, 0x09)));
    assert_eq!(block_sample(inputs[1]), IResult::Done(slice, (0x01, 0x0a)));
    assert_eq!(block_sample(inputs[2]), error(inputs[2]));
    assert_eq!(block_sample(inputs[3]), error(inputs[3]));
  }

  #[test]
  fn test_channel_bits() {
    let inputs  = [b"\x58", b"\x80", b"\xac", b"\xf2", b"\xae", b"\x91"];
    let slice   = &[][..];
    let results = [ IResult::Done(slice, (ChannelAssignment::Independent,
                                          6, 4))
                  , IResult::Done(slice, (ChannelAssignment::LeftSide, 2, 0))
                  , IResult::Done(slice, (ChannelAssignment::MidpointSide,
                                          2, 6))
                  ];

    assert_eq!(channel_bits(inputs[0]), results[0]);
    assert_eq!(channel_bits(inputs[1]), results[1]);
    assert_eq!(channel_bits(inputs[2]), results[2]);
    assert_eq!(channel_bits(inputs[3]), error(inputs[3]));
    assert_eq!(channel_bits(inputs[4]), error(inputs[4]));
    assert_eq!(channel_bits(inputs[5]), error(inputs[5]));
  }

  #[test]
  fn test_utf8_header() {
    let inputs  = [b"\x74", b"\xfc", b"\xfe", b"\xfe", b"\xff", b"\xff"];
    let slice   = &[][..];
    let results = [ IResult::Done(slice, Some((0, 116)))
                  , IResult::Done(slice, Some((5, 0)))
                  , IResult::Done(slice, None)
                  , IResult::Done(slice, Some((6, 0)))
                  , IResult::Done(slice, None)
                  , IResult::Done(slice, None)
                  ];

    assert_eq!(utf8_header(inputs[0], false), results[0]);
    assert_eq!(utf8_header(inputs[1], true), results[1]);
    assert_eq!(utf8_header(inputs[2], false), results[2]);
    assert_eq!(utf8_header(inputs[3], true), results[3]);
    assert_eq!(utf8_header(inputs[4], false), results[4]);
    assert_eq!(utf8_header(inputs[5], true), results[5]);
  }

  #[test]
  fn test_number_type() {
    let inputs  = [ &b"\xa0"[..], &b"\xaa\xaa"[..]
                  , &b"\x80\x80\x88\x80\x80"[..]
                  , &b"\xbf\x80\xbf\x80\xbf\x80"[..]
                  ];
    let slice   = &[][..];
    let results = [ IResult::Done(slice, NumberType::Frame(32))
                  , IResult::Done(slice, NumberType::Sample(43690))
                  , IResult::Done(slice, NumberType::Frame(32768))
                  , IResult::Done(slice, NumberType::Sample(67662254016))
                  ];

    assert_eq!(number_type(inputs[0], false, (1, 0x00)), results[0]);
    assert_eq!(number_type(inputs[1], true, (2, 0x0a)), results[1]);
    assert_eq!(number_type(inputs[2], false, (5, 0x00)), results[2]);
    assert_eq!(number_type(inputs[3], true, (6, 0x00)), results[3]);
  }

  #[test]
  fn test_secondary_block_size() {
    let inputs  = [&b"\x4b"[..], &b"\x01\0"[..]];
    let slice   = &[][..];
    let results = [ IResult::Done(slice, Some(75))
                  , IResult::Done(slice, Some(256))
                  , IResult::Done(slice, None)
                  ];

    assert_eq!(secondary_block_size(inputs[0], 0b0110), results[0]);
    assert_eq!(secondary_block_size(inputs[1], 0b0111), results[1]);
    assert_eq!(secondary_block_size(slice, 0b1111), results[2]);
  }

  #[test]
  fn test_secondary_sample_rate() {
    let inputs  = [&b"\x1a"[..], &b"\x10\x04"[..]];
    let slice   = &[][..];
    let results = [ IResult::Done(slice, Some(26))
                  , IResult::Done(slice, Some(4100))
                  , IResult::Done(slice, None)
                  ];

    assert_eq!(secondary_sample_rate(inputs[0], 0b1100), results[0]);
    assert_eq!(secondary_sample_rate(inputs[1], 0b1110), results[1]);
    assert_eq!(secondary_sample_rate(slice, 0b1111), results[2]);
  }

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
                       channels: 2,
                       channel_assignment: ChannelAssignment::Independent,
                       bits_per_sample: 24,
                       number: NumberType::Frame(65536),
                       crc: 0x2e,
                     })
                   , IResult::Done(&[][..], Header {
                       block_size: 512,
                       sample_rate: 1000,
                       channels: 2,
                       channel_assignment: ChannelAssignment::MidpointSide,
                       bits_per_sample: 16,
                       number: NumberType::Sample(68719476732),
                       crc: 0x88,
                     })
                   , IResult::Done(&[][..], Header {
                       block_size: 4096,
                       sample_rate: 32000,
                       channels: 8,
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
