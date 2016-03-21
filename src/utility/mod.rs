mod crc;
#[macro_use]
mod macros;
mod types;

pub use self::crc::{crc8, crc16};
pub use self::types::{ErrorKind, ByteStream, ReadStream};

use nom::{self, IResult};
use metadata::{Metadata, metadata_parser};

/// An interface for parsing through some type of producer to a byte stream.
///
/// External parsers get passed in and consumes the bytes held internally
/// and outputs the `Result` of that parser.
pub trait StreamProducer {
  fn parse<F, T>(&mut self, f: F) -> Result<T, ErrorKind>
   where F: FnOnce(&[u8]) -> IResult<&[u8], T, ErrorKind>;
}

pub trait Sample {
  type Normal;

  fn size() -> usize;
  fn to_normal(sample: Self) -> Option<Self::Normal>;
}

// Convert one to four byte slices into an unsigned 32-bit number.
//
// NOTE: This assumes big-endian since most numbers in the FLAC binary are
// that endianness.
#[inline]
pub fn to_u32(bytes: &[u8]) -> u32 {
  let length = bytes.len();

  debug_assert!(length <= 4);

  (0..length).fold(0, |result, i|
    result + ((bytes[i] as u32) << ((length - 1 - i) * 8))
  )
}

// Extends a signed value of a specific bit size to a larger bit size.
//
// NOTE: This assumes that the larger bit size will be 32 bit since that is
// the largest sample size supported in FLAC.
pub fn extend_sign(value: u32, bit_count: usize) -> i32 {
  if bit_count >= 32 || value < (1 << (bit_count - 1)) {
    value as i32
  } else {
    (value as i32).wrapping_sub(1 << bit_count)
  }
}

// Bit shifted version for two to the power of a given exponent.
#[inline]
pub fn power_of_two(exponent: u32) -> u32 {
  debug_assert!(exponent <= 31);

  1 << exponent
}

#[derive(PartialEq, Eq)]
enum ParserState {
  Header,
  StreamInfo,
  Metadata
}

fn parser<'a>(input: &'a [u8], state: &mut ParserState)
              -> IResult<&'a [u8], Metadata, ErrorKind> {
  let mut slice = input;
  let error     = nom::Err::Code(nom::ErrorKind::Custom(ErrorKind::Unknown));

  if *state == ParserState::Header {
    let (i, _) = try_parser! {
      to_custom_error!(slice, tag!("fLaC"), HeaderParser)
    };

    slice  = i;
    *state = ParserState::StreamInfo;
  }

  match *state {
    ParserState::StreamInfo => {
      let (i, block) = try_parse!(slice, metadata_parser);

      if block.is_stream_info() {
        *state = ParserState::Metadata;

        IResult::Done(i, block)
      } else {
        IResult::Error(error)
      }
    }
    ParserState::Metadata   => metadata_parser(slice),
    _                       => IResult::Error(error),
  }
}

pub fn many_metadata<S, F>(stream: &mut S, mut f: F) -> Result<(), ErrorKind>
 where S: StreamProducer,
       F: FnMut(Metadata) {
  let mut state  = ParserState::Header;
  let mut result = Ok(());

  loop {
    match stream.parse(|i| parser(i, &mut state)) {
      Ok(block)                => {
        let is_last = block.is_last();

        f(block);

        if is_last {
          break;
        }
      }
      Err(ErrorKind::Continue) => continue,
      Err(e)                   => {
        result = Err(e);

        break;
      }
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[should_panic]
  fn test_panic_to_u32() {
    to_u32(&[0x00, 0x00, 0x00, 0x00, 0x00]);
  }

  #[test]
  fn test_to_u32() {
    let bytes = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];

    assert_eq!(to_u32(&bytes[0..1]), 0x00000001);
    assert_eq!(to_u32(&bytes[3..5]), 0x00006789);
    assert_eq!(to_u32(&bytes[1..4]), 0x00234567);
    assert_eq!(to_u32(&bytes[4..]), 0x89abcdef);
  }

  #[test]
  fn test_extend_sign() {
    assert_eq!(extend_sign(32, 6), -32);
    assert_eq!(extend_sign(31, 6), 31);
    assert_eq!(extend_sign(128, 8), -128);
    assert_eq!(extend_sign(127, 8), 127);

    assert_eq!(extend_sign(2048, 12), -2048);
    assert_eq!(extend_sign(2047, 12), 2047);
    assert_eq!(extend_sign(32768, 16), -32768);
    assert_eq!(extend_sign(32767, 16), 32767);

    assert_eq!(extend_sign(8388608, 24), -8388608);
    assert_eq!(extend_sign(8388607, 24), 8388607);
    assert_eq!(extend_sign(2147483648, 32), -2147483648);
    assert_eq!(extend_sign(2147483647, 32), 2147483647);
  }

  #[test]
  #[should_panic]
  fn test_panic_power_of_two() {
    power_of_two(32);
  }

  #[test]
  fn test_power_of_two() {
    assert_eq!(power_of_two(0), 1);
    assert_eq!(power_of_two(1), 2);
    assert_eq!(power_of_two(2), 4);
    assert_eq!(power_of_two(10), 1024);
    assert_eq!(power_of_two(31), 2147483648);
  }
}
